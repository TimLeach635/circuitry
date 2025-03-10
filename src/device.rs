use std::collections::{HashMap, HashSet};

pub mod memory;
mod debugger;

type PortIdentifier = String;
type PortValue = u32;

/// Represents an error that can be thrown by methods in the [`Device`] trait.
///
/// Not yet fleshed out. In future these should be much more comprehensive.
#[derive(Debug)]
pub struct DeviceError;

/// Represents a device in a circuit.
pub trait Device {
    /// Get a [`HashSet`] containing the identifiers of all input ports on this device.
    fn get_input_ports(&self) -> HashSet<PortIdentifier>;

    /// Get a [`HashSet`] containing the identifiers of all input ports on this device.
    fn get_output_ports(&self) -> HashSet<PortIdentifier>;
    
    /// Provide a set of values for input ports to this device.
    /// 
    /// Should fail in the following circumstances:
    /// * A value is provided for a port whose value has already been provided
    /// * A value is provided for an output port (only input ports have values provided from
    ///   outside)
    /// * A value is provided for an unknown port
    /// 
    /// Should return a [`HashMap`] whose keys are the identifiers of output ports whose values are
    /// now known, and whose values are the values of those output ports. 
    fn provide_port_values(&mut self, values: HashMap<PortIdentifier, PortValue>)
        -> Result<HashMap<PortIdentifier, PortValue>, DeviceError>;
    
    /// Perform a tick. Should fail if this device has not had enough ports specified to know what
    /// to do this tick.
    fn tick(&mut self) -> Result<HashMap<PortIdentifier, PortValue>, DeviceError>;
}
