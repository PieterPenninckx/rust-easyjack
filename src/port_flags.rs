#![allow(dead_code)]

/// if JackPortIsInput is set, then the port can receive data.
pub const JACK_PORT_IS_INPUT: u64 = 0x1;

/// if JackPortIsOutput is set, then data can be read from the port.
pub const JACK_PORT_IS_OUTPUT: u64 = 0x2;

/// if JackPortIsPhysical is set, then the port corresponds to some kind of physical I/O
/// connector.
pub const JACK_PORT_IS_PHYSICAL: u64 = 0x4;

/// if JackPortCanMonitor is set, then a call to jack_port_request_monitor() makes sense.
///
/// Precisely what this means is dependent on the client. A typical result of it being called
/// with TRUE as the second argument is that data that would be available from an output port
/// (with JackPortIsPhysical set) is sent to a physical output connector as well, so that it
/// can be heard/seen/whatever.
///
/// Clients that do not control physical interfaces should never create ports with this bit
/// set.
pub const JACK_PORT_CAN_MONITOR : u64 = 0x8;

/// JackPortIsTerminal means:
///
///  * for an input port: the data received by the port will not be passed on or made available
///    at any other port
///
/// * for an output port: the data available at the port does not originate from any other port
///
/// Audio synthesizers, I/O hardware interface clients, HDR systems are examples of clients
/// that would set this flag for their ports.
pub const JACK_PORT_IS_TERMINAL: u64 = 0x10;
