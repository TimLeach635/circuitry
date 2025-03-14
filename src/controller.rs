use std::collections::{HashMap, HashSet};
use crate::device::{Device, PortIdentifier};

pub type DeviceIdentifier = String;

#[derive(Eq, Hash, PartialEq)]
struct DevicePort {
    device: DeviceIdentifier,
    port: PortIdentifier,
}

#[derive(Eq, Hash, PartialEq)]
struct Connection {
    from: DevicePort,
    to: DevicePort,
}

pub struct Controller {
    devices: HashMap<DeviceIdentifier, Box<dyn Device>>,
    ports: HashSet<DevicePort>,
    connections: HashSet<Connection>,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            devices: HashMap::new(),
            ports: HashSet::new(),
            connections: HashSet::new(),
        }
    }
    pub fn add_device(&mut self, name: DeviceIdentifier, device: Box<dyn Device>) {
        self.devices.insert(name, device);
    }
    
    pub fn add_connection(
        &mut self,
        from_device: DeviceIdentifier, from_port: PortIdentifier,
        to_device: DeviceIdentifier, to_port: PortIdentifier,
    ) {
        self.connections.insert(Connection {
            from: DevicePort {
                device: from_device,
                port: from_port,
            },
            to: DevicePort {
                device: to_device,
                port: to_port,
            }
        });
    }
    
    pub fn tick(&mut self) {
        
    }
}
