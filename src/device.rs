use std::collections::{HashMap, HashSet};

pub mod memory;
pub mod debugger;

pub type PortIdentifier = String;
pub type PortValue = u32;

/// Represents an error that can be thrown by methods in the [`Device`] trait.
///
/// Not yet fleshed out. In future these should be much more comprehensive.
#[derive(Debug)]
pub struct DeviceError;

/// Represents a device in a circuit.
/// 
/// More of a note to myself than anything (so if you're reading this, you're either me, or I forgot
/// to delete it): I'm going to write this under a couple of assumptions that I know are not
/// correct. I'm doing this because I've been prematurely optimising this for ages now, and I just
/// want to get _something_ down, but I can't bear to write code that I consider suboptimal... so
/// the only option is to simplify the problem.
/// 
/// **Incorrect simplifying assumption 1**: A device must have all of its input ports specified in
/// order to resolve every tick. This makes logic easier, but is a "sufficient but not necessary"
/// constraint; sometimes certain inputs don't need to be known for the device to know what to do,
/// and by enforcing this constraint we disallow circuit designs that make use of that.
/// 
/// **Incorrect simplifying assumption 2**: Output dependencies don't change. Again, this allows for
/// simplifying logic out on the controller level because we don't need to rebuild the dependency
/// graph, but it disallows more complicated circuit designs that would nonetheless be valid and
/// resolvable.
pub trait Device {
    /// Get a [`HashSet`] containing the identifiers of all input ports on this device.
    fn get_input_ports(&self) -> HashSet<PortIdentifier>;

    /// Get a [`HashSet`] containing the identifiers of all input ports on this device.
    fn get_output_ports(&self) -> HashSet<PortIdentifier>;
    
    /// Get a [`HashSet`] containing the identifiers of all input ports required to resolve the
    /// given output port.
    /// 
    /// Fails if:
    /// * The provided port is unknown
    /// * The provided port is not an output port
    /// 
    /// Following on from the (incorrect) assumptions in the description of [`Device`], this should
    /// return a complete set of the dependencies of the provided output; in other words, if all
    /// the input ports returned by this function are provided to the device, the output should be
    /// guaranteed to be known.
    fn get_output_dependencies(&self, output: &PortIdentifier)
        -> Result<HashSet<PortIdentifier>, DeviceError>;

    /// Provide a single value for an input port to this device.
    ///
    /// Should fail in the following circumstances:
    /// * A value is provided for a port whose value has already been provided
    /// * A value is provided for an output port (only input ports have values provided from
    ///   outside)
    /// * A value is provided for an unknown port
    fn provide_port_value(&mut self, port: PortIdentifier, value: PortValue)
        -> Result<(), DeviceError>;
    
    /// Provide a set of values for input ports to this device.
    /// 
    /// Should fail in the following circumstances:
    /// * A value is provided for a port whose value has already been provided
    /// * A value is provided for an output port (only input ports have values provided from
    ///   outside)
    /// * A value is provided for an unknown port
    fn provide_port_values(&mut self, values: HashMap<PortIdentifier, PortValue>)
        -> Result<(), DeviceError>;
    
    /// Get the value of the provided output port.
    /// 
    /// Fails (returns `Err(DeviceError)`) if:
    /// * The provided port is unknown
    /// * The provided port is not an output port
    /// 
    /// Returns `Ok(None)` if the output port's value is not yet known.
    /// 
    /// Returns `Ok(Some(port_value))` if the output port's value has resolved.
    fn get_port_value(&self, port: &PortIdentifier) -> Result<Option<PortValue>, DeviceError>;
    
    /// Perform a tick. Should fail if this device has not had enough ports specified to know what
    /// to do this tick.
    fn tick(&mut self) -> Result<(), DeviceError>;
}
