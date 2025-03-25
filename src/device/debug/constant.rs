use std::collections::{HashMap, HashSet};
use crate::device::{Device, DeviceError, PortIdentifier, PortValue};

pub struct Constant {
    output_port: PortIdentifier,
    value: PortValue,
}

impl Constant {
    pub fn new(
        output_port: PortIdentifier,
        value: PortValue,
    ) -> Constant {
        Constant {
            output_port,
            value,
        }
    }
}

impl Device for Constant {
    fn get_input_ports(&self) -> HashSet<PortIdentifier> {
        // No input ports
        HashSet::new()
    }

    fn get_output_ports(&self) -> HashSet<PortIdentifier> {
        let mut result = HashSet::new();
        result.insert(self.output_port.to_owned());
        result
    }

    fn get_output_dependencies(&self, output: &PortIdentifier) -> Result<HashSet<PortIdentifier>, DeviceError> {
        if *output != self.output_port {
            return Err(DeviceError);
        }
        Ok(HashSet::new())
    }

    fn provide_port_value(&mut self, _: PortIdentifier, _: PortValue)
        -> Result<(), DeviceError>
    {
        // No input ports, so this operation always fails
        Err(DeviceError)
    }

    fn provide_port_values(&mut self, _: HashMap<PortIdentifier, PortValue>)
        -> Result<(), DeviceError> {
        // No input ports, so this operation always fails
        Err(DeviceError)
    }

    fn get_port_value(&self, port: &PortIdentifier) -> Result<Option<PortValue>, DeviceError> {
        match *port == self.output_port {
            true => Ok(Some(self.value)),
            false => Err(DeviceError),
        }
    }

    fn tick(&mut self) -> Result<(), DeviceError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::device::{Device, PortIdentifier, PortValue};
    use crate::device::debug::constant::Constant;
    
    #[test]
    fn constant_can_be_instantiated() {
        _ = Constant::new("qq".to_owned(), 0);
    }
    
    #[test]
    fn constant_outputs_value_on_given_port() {
        let port: PortIdentifier = "qq".to_owned();
        let value: PortValue = 1;
        let debugger = Constant::new(port.clone(), value);
        
        let result = debugger.get_port_value(&port);
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, Some(value));
    }
    
    #[test]
    fn constant_always_outputs_provided_value() {
        let port: PortIdentifier = "qq".to_owned();
        let value: PortValue = 1;
        let mut debugger = Constant::new(port.clone(), value);
    
        for _ in 0..3 {
            let val = debugger.get_port_value(&port).unwrap();
            assert_eq!(val, Some(value));
            let _ = debugger.tick().unwrap();
        }
    }
}
