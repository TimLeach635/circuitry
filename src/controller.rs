use std::collections::HashMap;
use crate::device::{Device, PortIdentifier, PortValue};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::acyclic::{Acyclic};
use petgraph::Direction;

pub type DeviceIdentifier = String;

#[derive(Clone)]
enum EdgeType {
    Internal,
    External,
}

#[derive(Debug)]
pub struct ControllerError;

/// Holds devices and the connections between them, and facilitates whole-circuit ticks and
/// information flow.
pub struct Controller {
    /// Holds all the devices managed by this `Controller`, mapped by their identifier.
    devices: HashMap<DeviceIdentifier, Box<dyn Device>>,
    
    /// Holds references to all the ports known by this `Controller`. The stored value is the index
    /// of the port's node in the dependency graph.
    ports: HashMap<(DeviceIdentifier, PortIdentifier), NodeIndex>,
    
    /// Holds the dependency graph of all the ports on all the devices managed by this `Controller`.
    /// 
    /// Edges of this graph are either:
    /// * _external_, representing a user-defined connection from an output port of one device to
    ///   an input port of another; or
    /// * _internal_, representing an intra-device dependency where the output port of a device
    ///   depends on a particular input port _on the same device_.
    dependencies: DiGraph<(DeviceIdentifier, PortIdentifier), EdgeType>,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            devices: HashMap::new(),
            ports: HashMap::new(),
            dependencies: DiGraph::new(),
        }
    }
    pub fn add_device(&mut self, id: DeviceIdentifier, device: Box<dyn Device>) {
        // Add the ports in this device to the graph
        for input_port in device.get_input_ports().iter() {
            let index = self.dependencies.add_node(
                (id.clone(), input_port.clone())
            );
            self.ports.insert((id.clone(), input_port.clone()), index);

        }
        for output_port in device.get_output_ports().iter() {
            let index = self.dependencies.add_node(
                (id.clone(), output_port.clone())
            );
            self.ports.insert((id.clone(), output_port.clone()), index);

            // Add the internal dependencies as edges
            // Don't need to worry about cycles being introduced, as the nodes have no other
            // connections at this stage
            let port_deps = device.get_output_dependencies(output_port)
                .expect("Port identifiers returned by device's `get_output_ports()` method \
                should always be valid inputs to same device's `get_output_dependencies()` method");
            for dep in port_deps.iter() {
                let dep_idx = self.ports[&(id.clone(), dep.clone())];
                self.dependencies.add_edge(dep_idx, index, EdgeType::Internal);
            }
        }

        // Add the device
        // Do this after the ports because it moves the device into the HashMap
        self.devices.insert(id, device);
    }

    /// Attempt to add a connection between two ports known by this [`Controller`].
    /// Fails (returns `Err`) if:
    /// * Any of the devices or ports are not known by the controller
    /// * TODO: The "from" port is not an output port
    /// * TODO: The "to" port is not an input port
    /// * Adding the connection would result in the dependency graph containing a cycle
    pub fn add_connection(
        &mut self,
        from_device: &DeviceIdentifier, from_port: &PortIdentifier,
        to_device: &DeviceIdentifier, to_port: &PortIdentifier,
    ) -> Result<(), ControllerError> {
        // Get indices of ports
        let from_idx = match self.ports.get(&(from_device.clone(), from_port.clone())) {
            None => return Err(ControllerError),
            Some(idx) => *idx,
        };
        let to_idx = match self.ports.get(&(to_device.clone(), to_port.clone())) {
            None => return Err(ControllerError),
            Some(idx) => *idx,
        };

        // Wrap graph to enforce acyclic constraint
        // TODO: improve the memory usage of this, I don't like the clone
        let mut acyclic = Acyclic::try_from_graph(self.dependencies.clone())
            .expect("`self.dependencies` should never contain cycles, so should always be \
            wrappable in the `Acyclic` wrapper");
        match acyclic.try_add_edge(from_idx, to_idx, EdgeType::External) {
            Err(_) => return Err(ControllerError),
            _ => {},
        };
        self.dependencies = acyclic.into_inner();

        Ok(())
    }

    /// Perform a tick.
    ///
    /// This uses the dependency graph to figure out the value of every single port and connection
    /// in the circuit (returning the values of all ports as a [`HashSet`]).
    pub fn tick(&mut self)
        -> Result<HashMap<(DeviceIdentifier, PortIdentifier), PortValue>, ControllerError>
    {
        // Here is where the acyclic data structure comes into its own - we can perform a
        // "topological sort" of the nodes, which will tell us what order to traverse them in
        // TODO: improve the memory usage of this, I don't like the clone
        let acyclic = Acyclic::try_from_graph(self.dependencies.clone())
            .expect("`self.dependencies` should never contain cycles, so should always be \
            wrappable in the `Acyclic` wrapper");
        let mut result: HashMap<(DeviceIdentifier, PortIdentifier), PortValue> = HashMap::new();
        for port_idx in acyclic.nodes_iter() {
            let (device_id, port_id) = acyclic.node_weight(port_idx)
                .expect("Node index retrieved from `nodes_iter()` should return `Some` \
                from `node_weight()`");

            let device = self.devices.get(device_id)
                .expect("Device id retrieved from `node_weight()` should always be a key \
                present in `self.devices`");
            if device.get_output_ports().contains(port_id) {
                // This is an output port, so we should have provided its dependencies in a
                // previous iteration, or it has no dependencies
                result.insert(
                    (device_id.clone(), port_id.clone()),
                    device.get_port_value(port_id)
                        .expect("Port value retrieved from `get_output_ports()` should always \
                            be a valid input to `get_port_value()`")
                        .expect("Output port should have a known value at this point in the \
                            topological sort")
                );
            } else if device.get_input_ports().contains(port_id) {
                // This is an input port, so its value must be coming from a connected output port.
                // For now (and this will be changed), all input ports must have a value connected
                // to them, so if we don't find a connection between this and an output port, that
                // is an error.
                // Thanks to the `neighbors_directed()` function, we can just directly find the
                // connected output port, as it will be at the other end of the only incoming
                // edge to this input port.
                let mut incoming_neighbours = acyclic.neighbors_directed(
                    port_idx,
                    Direction::Incoming
                );
                let n_incoming = incoming_neighbours.clone().count();
                let output_port_idx = match n_incoming {
                    0 => return Err(ControllerError),  // No incoming value, cannot resolve
                    1 => incoming_neighbours.next().expect("Length is 1, so `next()` should \
                        return a value"),
                    _ => panic!("Should not end up in a state where an input port can have more \
                        than one incoming connection"),
                };

                // Get the value of that other port
                let (output_device_id, output_port_id) = acyclic
                    .node_weight(output_port_idx)
                    .expect("Node index retrieved from `neighbors_directed()` should return \
                    `Some` from `node_weight()`");
                let output_device = self.devices.get(output_device_id)
                    .expect("Device id retrieved from `node_weight()` should always be a key \
                    present in `self.devices`");
                let value = output_device.get_port_value(output_port_id)
                    .expect("Port connected to an input port should be an output port, and \
                        should be present on the device arrived at at this point")
                    .expect("Output port should have a known value at this point in the \
                            topological sort");

                // Pass it to this device
                // Need to re-borrow it as mutable
                let device = self.devices.get_mut(device_id)
                    .expect("Using same device id as before should retrieve value");
                device.provide_port_value(port_id.clone(), value).expect("Port id and value \
                    obtained at this point should be valid inputs to `provide_port_value()`");

                // Store this value
                result.insert((device_id.clone(), port_id.clone()), value);
            }
        }

        // TODO: Ensure that if a failure occurs in one of the loops, we rollback the provision of
        //  values to the other devices so a tick can be attempted again

        // Now that we have retrieved all the values, we should perform a tick on every device
        for device in self.devices.values_mut() {
            if device.tick().is_err() {
                // TODO: Rollback ticks so this can be done again
                return Err(ControllerError);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::controller::{Controller, DeviceIdentifier};
    use crate::device::debug::constant::Constant;
    use crate::device::debug::sequencer::Sequencer;
    use crate::device::memory::Memory;
    use crate::device::{PortIdentifier, PortValue};

    #[test]
    fn controller_can_be_instantiated() {
        let _ = Controller::new();
    }
    
    #[test]
    fn controller_can_have_devices_added_to_it() {
        let mut controller = Controller::new();
        let memory = Memory::new();
        let sequencer = Sequencer::new("qq".to_owned(), &vec![0]).unwrap();
        
        let _ = controller.add_device("Memory".to_owned(), Box::new(memory));
        let _ = controller.add_device("Sequencer".to_owned(), Box::new(sequencer));
    }
    
    #[test]
    fn controller_can_have_connections_added_to_it() {
        let mut controller = Controller::new();
        let memory = Memory::new();
        let sequencer = Sequencer::new("qq".to_owned(), &vec![0]).unwrap();

        let _ = controller.add_device("Memory".to_owned(), Box::new(memory));
        let _ = controller.add_device("Sequencer".to_owned(), Box::new(sequencer));
        
        let result = controller.add_connection(
            &"Sequencer".to_owned(), &"qq".to_owned(),
            &"Memory".to_owned(), &"ra".to_owned(),
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn controller_cannot_have_cyclic_connections_added_to_it() {
        let mut controller = Controller::new();
        let memory = Memory::new();

        _ = controller.add_device("Memory".to_owned(), Box::new(memory));

        let result = controller.add_connection(
            &"Memory".to_owned(), &"rv".to_owned(),
            &"Memory".to_owned(), &"ra".to_owned(),
        );

        assert!(result.is_err());
    }
    
    #[test]
    fn controller_can_perform_tick_with_no_devices() {
        let mut controller = Controller::new();
        let result = controller.tick();
        assert!(result.is_ok());
    }
    
    #[test]
    fn controller_can_perform_tick_with_device_with_only_outputs() {
        let mut controller = Controller::new();
        let constant = Constant::new("qq".to_owned(), 1);

        _ = controller.add_device("Constant".to_owned(), Box::new(constant));
        
        let result = controller.tick();
        assert!(result.is_ok());
    }

    #[test]
    fn controller_gets_output_values_on_tick() {
        let mut controller = Controller::new();
        let device_id: DeviceIdentifier = "Constant".to_owned();
        let port_id: PortIdentifier = "qq".to_owned();
        let value: PortValue = 1;
        let constant = Constant::new(port_id.clone(), value);

        _ = controller.add_device(device_id.clone(), Box::new(constant));

        let result = controller.tick().unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&(device_id.clone(), port_id.clone())));
        assert_eq!(result.get(&(device_id.clone(), port_id.clone())), Some(&value));
    }
    
    #[test]
    fn controller_cannot_tick_with_unconnected_inputs() {
        let mut controller = Controller::new();
        let memory = Memory::new();

        _ = controller.add_device("Memory".to_owned(), Box::new(memory));

        let result = controller.tick();
        assert!(result.is_err());
    }

    #[test]
    fn controller_can_tick_with_connected_inputs() {
        let mut controller = Controller::new();
        let memory = Memory::new();
        let ra_const = Constant::new("qq".to_owned(), 1);
        let we_const = Constant::new("qq".to_owned(), 1);
        let wa_const = Constant::new("qq".to_owned(), 2);
        let wv_const = Constant::new("qq".to_owned(), 3);

        // Add devices
        _ = controller.add_device("Memory".to_owned(), Box::new(memory));
        _ = controller.add_device("RA constant".to_owned(), Box::new(ra_const));
        _ = controller.add_device("WE constant".to_owned(), Box::new(we_const));
        _ = controller.add_device("WA constant".to_owned(), Box::new(wa_const));
        _ = controller.add_device("WV constant".to_owned(), Box::new(wv_const));
        
        // Add connections
        _ = controller.add_connection(
            &"RA constant".to_owned(), &"qq".to_owned(),
            &"Memory".to_owned(), &"ra".to_owned(),
        ).unwrap();
        _ = controller.add_connection(
            &"WE constant".to_owned(), &"qq".to_owned(),
            &"Memory".to_owned(), &"we".to_owned(),
        ).unwrap();
        _ = controller.add_connection(
            &"WA constant".to_owned(), &"qq".to_owned(),
            &"Memory".to_owned(), &"wa".to_owned(),
        ).unwrap();
        _ = controller.add_connection(
            &"WV constant".to_owned(), &"qq".to_owned(),
            &"Memory".to_owned(), &"wv".to_owned(),
        ).unwrap();

        let result = controller.tick();
        assert!(result.is_ok());
    }
    
    #[test]
    fn controller_passes_values_and_performs_ticks_on_devices() {
        let mut controller = Controller::new();
        let memory = Memory::new();
        let ra_const = Constant::new("qq".to_owned(), 1);
        let we_const = Constant::new("qq".to_owned(), 1);
        let wa_const = Constant::new("qq".to_owned(), 1);
        let written_value: PortValue = 5;
        let wv_const = Constant::new("qq".to_owned(), written_value);

        // Add devices
        _ = controller.add_device("Memory".to_owned(), Box::new(memory));
        _ = controller.add_device("RA constant".to_owned(), Box::new(ra_const));
        _ = controller.add_device("WE constant".to_owned(), Box::new(we_const));
        _ = controller.add_device("WA constant".to_owned(), Box::new(wa_const));
        _ = controller.add_device("WV constant".to_owned(), Box::new(wv_const));

        // Add connections
        _ = controller.add_connection(
            &"RA constant".to_owned(), &"qq".to_owned(),
            &"Memory".to_owned(), &"ra".to_owned(),
        ).unwrap();
        _ = controller.add_connection(
            &"WE constant".to_owned(), &"qq".to_owned(),
            &"Memory".to_owned(), &"we".to_owned(),
        ).unwrap();
        _ = controller.add_connection(
            &"WA constant".to_owned(), &"qq".to_owned(),
            &"Memory".to_owned(), &"wa".to_owned(),
        ).unwrap();
        _ = controller.add_connection(
            &"WV constant".to_owned(), &"qq".to_owned(),
            &"Memory".to_owned(), &"wv".to_owned(),
        ).unwrap();
        
        // After the first tick, memory reads 0 because it hasn't been written to yet
        let result = controller.tick().unwrap();
        assert!(result.contains_key(&("Memory".to_owned(), "rv".to_owned())));
        assert_eq!(result.get(&("Memory".to_owned(), "rv".to_owned())), Some(&0));
        
        // After the second tick, it should read 1 because we have written that value to it
        let result = controller.tick().unwrap();
        assert!(result.contains_key(&("Memory".to_owned(), "rv".to_owned())));
        assert_eq!(result.get(&("Memory".to_owned(), "rv".to_owned())), Some(&written_value));
    }
}
