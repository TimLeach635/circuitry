use std::collections::HashMap;
use crate::device::{Device, PortIdentifier};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::acyclic::{Acyclic};

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
}

#[cfg(test)]
mod tests {
    use crate::controller::Controller;
    use crate::device::debugger::Debugger;
    use crate::device::memory::Memory;

    #[test]
    fn controller_can_be_instantiated() {
        let _ = Controller::new();
    }
    
    #[test]
    fn controller_can_have_devices_added_to_it() {
        let mut controller = Controller::new();
        let memory = Memory::new();
        let debugger = Debugger::new("qq".to_owned(), &vec![0]).unwrap();
        
        let _ = controller.add_device("Memory".to_owned(), Box::new(memory));
        let _ = controller.add_device("Debugger".to_owned(), Box::new(debugger));
    }
    
    #[test]
    fn controller_can_have_connections_added_to_it() {
        let mut controller = Controller::new();
        let memory = Memory::new();
        let debugger = Debugger::new("qq".to_owned(), &vec![0]).unwrap();

        let _ = controller.add_device("Memory".to_owned(), Box::new(memory));
        let _ = controller.add_device("Debugger".to_owned(), Box::new(debugger));
        
        let result = controller.add_connection(
            &"Debugger".to_owned(), &"qq".to_owned(),
            &"Memory".to_owned(), &"ra".to_owned(),
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn controller_cannot_have_cyclic_connections_added_to_it() {
        let mut controller = Controller::new();
        let memory = Memory::new();

        let _ = controller.add_device("Memory".to_owned(), Box::new(memory));

        let result = controller.add_connection(
            &"Memory".to_owned(), &"rv".to_owned(),
            &"Memory".to_owned(), &"ra".to_owned(),
        );

        assert!(result.is_err());
    }
}
