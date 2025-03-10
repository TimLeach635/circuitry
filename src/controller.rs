use std::collections::HashMap;
use crate::device::Device;

pub struct Controller {
    devices: HashMap<i32, Box<dyn Device>>,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            devices: HashMap::new(),
        }
    }
    pub fn add_device(&mut self, id: i32, device: Box<dyn Device>) {
        self.devices.insert(id, device);
    }
}
