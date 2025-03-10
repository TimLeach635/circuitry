use std::collections::{HashMap, HashSet};
use crate::device::{Device, DeviceError, PortIdentifier, PortValue};

pub struct Debugger {
    output_port: PortIdentifier,
    values: Vec<PortValue>,
    current_value_idx: usize,
}

impl Debugger {
    pub fn new(
        output_port: PortIdentifier,
        values: &[PortValue],
    ) -> Result<Debugger, ()> {
        match values.is_empty() {
            true => Err(()),
            false => Ok(Debugger {
                output_port,
                values: values.to_owned(),
                current_value_idx: 0,
            })
        }
    }
}

impl Device for Debugger {
    fn get_input_ports(&self) -> HashSet<PortIdentifier> {
        // No input ports
        HashSet::new()
    }

    fn get_output_ports(&self) -> HashSet<PortIdentifier> {
        let mut result = HashSet::new();
        result.insert(self.output_port.to_owned());
        result
    }

    fn provide_port_values(&mut self, _: HashMap<PortIdentifier, PortValue>)
        -> Result<HashMap<PortIdentifier, PortValue>, DeviceError> {
        // No input ports, so this operation always fails
        Err(DeviceError)
    }

    fn tick(&mut self) -> Result<HashMap<PortIdentifier, PortValue>, DeviceError> {
        // Output the currently-pointed-to value, update the pointer, then exit
        let mut result = HashMap::new();
        result.insert(self.output_port.to_owned(), self.values[self.current_value_idx]);
        self.current_value_idx += 1;
        if self.current_value_idx >= self.values.len() {
            self.current_value_idx = 0;
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::device::debugger::Debugger;
    use crate::device::{Device, PortIdentifier, PortValue};

    #[test]
    fn debugger_cannot_be_instantiated_if_no_values_given() {
        let result = Debugger::new("qq".to_owned(), &vec![]);
        assert!(result.is_err());
    }
    
    #[test]
    fn debugger_can_be_instantiated() {
        let result = Debugger::new("qq".to_owned(), &vec![0]);
        assert!(result.is_ok());
    }
    
    #[test]
    fn debugger_outputs_values_on_given_port() {
        let port: PortIdentifier = "qq".to_owned();
        let value: PortValue = 1;
        let mut debugger = Debugger::new(port.clone(), &vec![value]).unwrap();
        
        let result = debugger.tick();
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.get(&port).is_some());
        assert_eq!(result.get(&port), Some(&value));
    }
    
    #[test]
    fn debugger_outputs_provided_values_in_order() {
        let port: PortIdentifier = "qq".to_owned();
        let values: Vec<PortValue> = vec![1, 2, 3];
        let mut debugger = Debugger::new(port.clone(), &values).unwrap();

        for idx in 0..3 {
            let result = debugger.tick().unwrap();
            let value = *result.get(&port).unwrap();
            assert_eq!(value, values[idx]);
        }
    }

    #[test]
    fn debugger_loops_when_end_of_values_reached() {
        let port: PortIdentifier = "qq".to_owned();
        let values: Vec<PortValue> = vec![1, 2, 3];
        let mut debugger = Debugger::new(port.clone(), &values).unwrap();

        let expected: Vec<PortValue> = vec![1, 2, 3, 1, 2, 3, 1, 2, 3];
        for idx in 0..9 {
            let result = debugger.tick().unwrap();
            let value = *result.get(&port).unwrap();
            assert_eq!(value, expected[idx]);
        }
    }
}
