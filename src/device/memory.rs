use std::collections::{HashMap, HashSet};
use crate::device::{Device, DeviceError, PortIdentifier, PortValue};

pub struct Memory {
    data: HashMap<u32, u32>,
    specified_this_tick: HashMap<PortIdentifier, PortValue>,
    in_ports: HashSet<PortIdentifier>,
    out_ports: HashSet<PortIdentifier>,
}

impl Memory {
    pub fn new() -> Memory {
        // Thought - make it a bit more restricted (and closer to real life) by having only one
        // address input, so you cannot read one address while writing another?
        let mut in_ports: HashSet<PortIdentifier> = HashSet::new();
        in_ports.insert("ra".to_owned());  // Read address
        in_ports.insert("we".to_owned());  // Write enable
        in_ports.insert("wa".to_owned());  // Write address
        in_ports.insert("wv".to_owned());  // Write value

        let mut out_ports: HashSet<PortIdentifier> = HashSet::new();
        out_ports.insert("rv".to_owned());  // Read value
        
        Memory {
            data: HashMap::new(),
            specified_this_tick: HashMap::new(),
            in_ports,
            out_ports,
        }
    }
}

impl Device for Memory {
    // I have written a lot of stuff in here that will be common to all devices, but I don't yet
    // know the best way of commonising them.
    // I just want to get something working, however, so I'm leaving that particular problem until
    // later.
    fn get_input_ports(&self) -> HashSet<PortIdentifier> {
        self.in_ports.to_owned()
    }

    fn get_output_ports(&self) -> HashSet<PortIdentifier> {
        self.out_ports.to_owned()
    }

    fn get_output_dependencies(&self, output: &PortIdentifier) -> Result<HashSet<PortIdentifier>, DeviceError> {
        if output.as_str() != "rv" {
            return Err(DeviceError);
        }
        Ok(HashSet::from_iter(vec!["ra".to_owned()]))
    }

    fn provide_port_values(&mut self, values: HashMap<PortIdentifier, PortValue>)
        -> Result<(), DeviceError> {
        // Check for problems
        for port in values.keys() {
            if self.specified_this_tick.contains_key(port)
            || !self.in_ports.contains(port) {
                return Err(DeviceError)
            }
        }
        
        // Do stuff
        let mut result: HashMap<PortIdentifier, PortValue> = HashMap::new();
        for port in values.keys() {
            self.specified_this_tick.insert(port.clone(), *values.get(port).unwrap());
            match port.as_str() {
                "ra" => {
                    result.insert(
                        "rv".to_owned(),
                        *self.data
                            .get(values.get(port).unwrap())
                            .unwrap_or(&0u32)
                    );
                }
                // None of the other ports affect the output value
                _ => {}
            }
        }
        Ok(())
    }

    fn get_port_value(&self, port: &PortIdentifier) -> Result<Option<PortValue>, DeviceError> {
        if port.as_str() != "rv" {
            return Err(DeviceError);
        }
        
        Ok(match self.specified_this_tick.get("ra") {
            None => None,
            Some(addr) => Some(*self.data.get(addr).unwrap_or(&0u32)),
        })
    }

    fn tick(&mut self) -> Result<(), DeviceError> {
        if !self.specified_this_tick.contains_key("we") {
            // Need to know if we are writing to memory this tick         
            return Err(DeviceError);
        }
        if *self.specified_this_tick.get("we").unwrap() != 0 {
            // We are writing, so we need to know the address and value
            if !self.specified_this_tick.contains_key("wa")
                || !self.specified_this_tick.contains_key("wv") {      
                return Err(DeviceError);
            }
            self.data.insert(
                *self.specified_this_tick.get("wa").unwrap(),
                *self.specified_this_tick.get("wv").unwrap()
            );
        }
        
        // Only output port is "rv", and it's not known at the beginning of a tick, so the return
        // is always empty
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::device::{Device, PortIdentifier, PortValue};
    use crate::device::memory::Memory;

    #[test]
    fn memory_can_be_instantiated() {
        let _ = Memory::new();
    }
    
    #[test]
    fn memory_cannot_have_unknown_ports_specified() {
        let mut memory = Memory::new();
        let mut ports: HashMap<PortIdentifier, PortValue> = HashMap::new();
        ports.insert("qq".to_owned(), 0);
        let result = memory.provide_port_values(ports);
        assert!(result.is_err());
    }

    #[test]
    fn memory_cannot_have_output_ports_specified() {
        let mut memory = Memory::new();
        let mut ports: HashMap<PortIdentifier, PortValue> = HashMap::new();
        ports.insert("rv".to_owned(), 0);
        let result = memory.provide_port_values(ports);
        assert!(result.is_err());
    }

    #[test]
    fn memory_cannot_have_ports_specified_twice() {
        let mut memory = Memory::new();
        let mut ports: HashMap<PortIdentifier, PortValue> = HashMap::new();
        ports.insert("ra".to_owned(), 0);
        
        // First time
        let result = memory.provide_port_values(ports.to_owned());
        assert!(result.is_ok());

        // Second time
        let result = memory.provide_port_values(ports);
        assert!(result.is_err());
    }
    
    #[test]
    fn memory_does_not_resolve_if_write_enable_not_given() {
        let mut memory = Memory::new();
        let result = memory.tick();
        assert!(result.is_err());
    }
    
    #[test]
    fn memory_resolves_if_write_enable_is_zero_and_other_write_ports_not_given() {
        let mut memory = Memory::new();
        let mut ports: HashMap<PortIdentifier, PortValue> = HashMap::new();
        ports.insert("we".to_owned(), 0);
        let _ = memory.provide_port_values(ports).unwrap();
        let result = memory.tick();
        assert!(result.is_ok());
    }

    #[test]
    fn memory_does_not_resolve_if_write_enable_is_nonzero_and_other_write_ports_not_given() {
        let mut memory = Memory::new();
        let mut ports: HashMap<PortIdentifier, PortValue> = HashMap::new();
        ports.insert("we".to_owned(), 1);
        let _ = memory.provide_port_values(ports).unwrap();
        let result = memory.tick();
        assert!(result.is_err());
    }
    
    #[test]
    fn memory_can_be_written_to() {
        let address: PortValue = 2;
        let value: PortValue = 3;
        let mut memory = Memory::new();
        let mut ports: HashMap<PortIdentifier, PortValue> = HashMap::new();
        ports.insert("we".to_owned(), 1);
        ports.insert("wa".to_owned(), address);
        ports.insert("wv".to_owned(), value);
        
        let _ = memory.provide_port_values(ports).unwrap();
        let result = memory.tick();
        assert!(result.is_ok());
        assert_eq!(memory.data.get(&address).unwrap(), &value);
    }

    #[test]
    fn memory_can_be_read() {
        let address: PortValue = 2;
        let value: PortValue = 3;
        let mut memory = Memory::new();
        memory.data.insert(address, value);
        let mut ports: HashMap<PortIdentifier, PortValue> = HashMap::new();
        ports.insert("ra".to_owned(), address);
    
        memory.provide_port_values(ports).unwrap();
        let result = memory.get_port_value(&"rv".to_owned());
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), value);
    }
}
