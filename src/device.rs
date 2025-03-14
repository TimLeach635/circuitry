use std::collections::{HashMap, HashSet};

pub mod memory;
mod debugger;

pub type PortIdentifier = String;
pub type PortValue = u32;

/// An enum representing the possible states of an output port.
/// 
/// It can either be `Unknown`, in which case it contains a [`HashSet`] of [`PortIdentifier`]s -
/// this is the set of ports that need to be provided for the value of this port to resolve;
/// or `Known`, in which case it contains the value of the port.
pub enum OutputPortState {
    Unknown(HashSet<PortIdentifier>),
    Known(PortValue),
}

/// Represents an error that can be thrown by methods in the [`Device`] trait.
///
/// Not yet fleshed out. In future these should be much more comprehensive.
#[derive(Debug)]
pub struct DeviceError;

/// Represents a device in a circuit.
pub trait Device {
    /// Get a [`HashSet`] containing the identifiers of all input ports on this device.
    fn get_input_ports(&self) -> HashSet<PortIdentifier>;

    /// Get a [`HashSet`] containing the identifiers of all output ports on this device.
    fn get_output_ports(&self) -> HashSet<PortIdentifier>;
    
    /// Get a [`HashSet`] containing the identifiers of the input ports whose values need to be
    /// provided in order for the device to resolve the value of the provided output port.
    /// 
    /// Should return an error if the provided output identifier does not correspond to a known
    /// output port.
    /// 
    /// Note: the output port is _not_ guaranteed to be known after the returned identifiers' ports'
    /// values are provided to the device. This is because it is possible for an output port to
    /// depend on different inputs _conditional on the value of other inputs._
    /// 
    /// Consider as a classic example a hypothetical read-only memory device. It has three ports:
    /// * `re` - "Read enable", an input port
    /// * `ra` - "Read address", an input port
    /// * `rv` - "Read value", an output port.
    /// 
    /// We imagine the behaviour of `re` to be as follows:
    /// * If `re` has value 0, `rv` has no value, or its value is always 0
    /// * If `re` has nonzero value, `rv` is set to the value of the memory at location `ra`
    /// 
    /// This has the result that `rv` depends on the value of `ra` _conditionally_ on the value of
    /// `re`.
    /// 
    /// So, the first time you call `get_inputs_required_for_output("rv".to_owned())` on this
    /// memory device, it would return a [`HashSet`] containing `re`.
    /// 
    /// If you then provide `re` with the value 0, the device can resolve the value of `rv` and
    /// another call to `get_inputs_required_for_output` would return an empty set.
    /// 
    /// If, however, you provide `re` with a nonzero value, `rv` cannot be resolved, and a
    /// subsequent call to `get_inputs_required_for_output("rv".to_owned())` would now return a
    /// [`HashSet`] containing only `ra`.
    /// 
    /// This behaviour allows the user more freedom in how they construct their circuits - by
    /// only asking for values that the device is _certain_ it needs, more complicated looping
    /// circuits can be constructed.
    fn get_inputs_required_for_output(&self, output: &PortIdentifier)
        -> Result<HashSet<PortIdentifier>, DeviceError>;

    /// Get a [`HashSet`] containing the identifiers of the input ports whose values need to be
    /// provided in order for the device to resolve the value of the provided output port.
    /// 
    /// Similarly to the related function, [`get_inputs_required_for_output`], it is not guaranteed
    /// that this device will be ready to perform a tick after the returned ports have their values
    /// provided, for the same reason.
    /// 
    /// However, unlike that function, this function does not return a [`Result`], instead always
    /// returning the [`HashSet`] - this is because it does not take any inputs, and so cannot
    /// result in an error.
    fn get_inputs_required_for_tick(&self)
        -> HashSet<PortIdentifier>;
    
    /// Get a [`HashMap`] containing the current values of the output ports on this device.
    fn get_output_port_values(&self) -> HashMap<PortIdentifier, OutputPortState>;
    
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
