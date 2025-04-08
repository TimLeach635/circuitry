use std::collections::{HashMap, HashSet};
use crate::device::{Device, DeviceError, PortIdentifier, PortValue};

#[derive(Eq, Hash, PartialEq)]
pub enum RegisterIdentifier {
    InstructionPointer,
    X,
    Y,
    Z,
}

pub enum SourceIdentifier {
    Immediate(PortValue),
    Register(RegisterIdentifier),
    Port(PortIdentifier),
}

pub enum DestinationIdentifier {
    Register(RegisterIdentifier),
    Port(PortIdentifier),
}

pub enum Instruction {
    Noop,
    Sleep,
    Move {
        source: SourceIdentifier,
        destination: DestinationIdentifier,
    },
    Increment(RegisterIdentifier),
    Decrement(RegisterIdentifier),
    Add {
        source_a: SourceIdentifier,
        source_b: SourceIdentifier,
        destination: DestinationIdentifier,
    },
    Subtract {
        source_a: SourceIdentifier,
        source_b: SourceIdentifier,
        destination: DestinationIdentifier,
    },
    // TODO: jumps/branches (need to add flags)
}

pub struct Cpu {
    instructions: Vec<Instruction>,
    specified_this_tick: HashMap<PortIdentifier, PortValue>,
    in_ports: HashSet<PortIdentifier>,
    out_ports: HashSet<PortIdentifier>,
    registers: HashMap<RegisterIdentifier, PortValue>,
}

impl Cpu {
    pub fn new() -> Cpu {
        Self::with_instructions(Vec::new())
    }
    
    pub fn with_instructions(instructions: Vec<Instruction>) -> Cpu {
        let mut in_ports: HashSet<PortIdentifier> = HashSet::new();
        in_ports.insert("ia".to_owned());  // Input A
        in_ports.insert("ib".to_owned());  // Input B
        in_ports.insert("ic".to_owned());  // Input C

        let mut out_ports: HashSet<PortIdentifier> = HashSet::new();
        out_ports.insert("oa".to_owned());  // Output A
        out_ports.insert("ob".to_owned());  // Output B
        out_ports.insert("oc".to_owned());  // Output C

        let mut registers: HashMap<RegisterIdentifier, PortValue> = HashMap::new();
        registers.insert(RegisterIdentifier::InstructionPointer, 0);
        registers.insert(RegisterIdentifier::X, 0);
        registers.insert(RegisterIdentifier::Y, 0);
        registers.insert(RegisterIdentifier::Z, 0);

        Cpu {
            instructions,
            specified_this_tick: HashMap::new(),
            in_ports,
            out_ports,
            registers,
        }
    }
    
    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
}

impl Device for Cpu {
    fn get_input_ports(&self) -> HashSet<PortIdentifier> {
        self.in_ports.to_owned()
    }

    fn get_output_ports(&self) -> HashSet<PortIdentifier> {
        self.out_ports.to_owned()
    }

    fn get_output_dependencies(&self, output: &PortIdentifier)
        -> Result<HashSet<PortIdentifier>, DeviceError>
    {
        match output.as_str() {
            "oa" | "ob" | "oc" => Ok(HashSet::from_iter(vec![
                "ia".to_owned(),
                "ib".to_owned(),
                "ic".to_owned(),
            ])),
            _ => Err(DeviceError),
        }
    }

    fn provide_port_value(&mut self, port: PortIdentifier, value: PortValue)
        -> Result<(), DeviceError>
    {
        todo!()
    }

    fn provide_port_values(&mut self, values: HashMap<PortIdentifier, PortValue>)
        -> Result<(), DeviceError>
    {
        todo!()
    }

    fn get_port_value(&self, port: &PortIdentifier) -> Result<Option<PortValue>, DeviceError> {
        todo!()
    }

    fn tick(&mut self) -> Result<(), DeviceError> {
        todo!()
    }
}
