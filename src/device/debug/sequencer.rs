use std::collections::{HashMap, HashSet};
use crate::device::{Device, DeviceError, PortIdentifier, PortValue};

pub struct Sequencer {
    output_port: PortIdentifier,
    values: Vec<PortValue>,
    current_value_idx: usize,
}

impl Sequencer {
    pub fn new(
        output_port: PortIdentifier,
        values: &[PortValue],
    ) -> Result<Sequencer, ()> {
        match values.is_empty() {
            true => Err(()),
            false => Ok(Sequencer {
                output_port,
                values: values.to_owned(),
                current_value_idx: 0,
            })
        }
    }
}

impl Device for Sequencer {
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

    fn provide_port_value(&mut self, port: PortIdentifier, value: PortValue)
        -> Result<(), DeviceError>
    {
        let mut values = HashMap::new();
        values.insert(port, value);
        self.provide_port_values(values)
    }

    fn provide_port_values(&mut self, _: HashMap<PortIdentifier, PortValue>)
        -> Result<(), DeviceError> {
        // No input ports, so this operation always fails
        Err(DeviceError)
    }

    fn get_port_value(&self, port: &PortIdentifier) -> Result<Option<PortValue>, DeviceError> {
        match *port == self.output_port {
            true => Ok(self.values.get(self.current_value_idx).cloned()),
            false => Err(DeviceError),
        }
    }

    fn tick(&mut self) -> Result<(), DeviceError> {
        self.current_value_idx += 1;
        if self.current_value_idx >= self.values.len() {
            self.current_value_idx = 0;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::device::debug::sequencer::Sequencer;
    use crate::device::{Device, PortIdentifier, PortValue};

    #[test]
    fn debugger_cannot_be_instantiated_if_no_values_given() {
        let result = Sequencer::new("qq".to_owned(), &vec![]);
        assert!(result.is_err());
    }
    
    #[test]
    fn debugger_can_be_instantiated() {
        let result = Sequencer::new("qq".to_owned(), &vec![0]);
        assert!(result.is_ok());
    }
    
    #[test]
    fn debugger_outputs_values_on_given_port() {
        let port: PortIdentifier = "qq".to_owned();
        let value: PortValue = 1;
        let debugger = Sequencer::new(port.clone(), &vec![value]).unwrap();
        
        let result = debugger.get_port_value(&port);
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, Some(value));
    }
    
    #[test]
    fn debugger_outputs_provided_values_in_order() {
        let port: PortIdentifier = "qq".to_owned();
        let values: Vec<PortValue> = vec![1, 2, 3];
        let mut debugger = Sequencer::new(port.clone(), &values).unwrap();
    
        for idx in 0..3 {
            let value = debugger.get_port_value(&port).unwrap();
            assert_eq!(value, Some(values[idx]));
            let _ = debugger.tick().unwrap();
        }
    }
    
    #[test]
    fn debugger_loops_when_end_of_values_reached() {
        let port: PortIdentifier = "qq".to_owned();
        let values: Vec<PortValue> = vec![1, 2, 3];
        let mut debugger = Sequencer::new(port.clone(), &values).unwrap();
    
        let expected: Vec<PortValue> = vec![1, 2, 3, 1, 2, 3, 1, 2, 3];
        for idx in 0..9 {
            let value = debugger.get_port_value(&port).unwrap();
            assert_eq!(value, Some(expected[idx]));
            let _ = debugger.tick().unwrap();
        }
    }
}
