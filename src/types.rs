use jack_sys;
pub type PortType = &'static str;
pub type NumFrames = jack_sys::jack_nframes_t;

/// This module contains constants and a bitflags! generated struct mapping to the jack port flags
/// bitset for specifying options on jack ports
///
/// TODO provide good example here
pub mod port_flags {
use jack_sys;

bitflags! {
    pub flags PortFlags: u32 {
        /// if PortIsInput is set, then the port can receive data.
        const PORT_IS_INPUT = jack_sys::JackPortIsInput,

        /// if PortIsOutput is set, then data can be read from the port.
        const PORT_IS_OUTPUT = jack_sys::JackPortIsOutput,

        /// if PortIsPhysical is set, then the port corresponds to some kind of physical I/O
        /// connector.
        const PORT_IS_PHYSICAL = jack_sys::JackPortIsPhysical,

        /// if PortCanMonitor is set, then a call to jack_port_request_monitor() makes sense.
        /// Precisely what this means is dependent on the client. A typical result of it being
        /// called with TRUE as the second argument is that data that would be available from an
        /// output port (with JackPortIsPhysical set) is sent to a physical output connector as
        /// well, so that it can be heard/seen/whatever.
        ///
        /// Clients that do not control physical interfaces should never create ports with this bit
        /// set.
        const PORT_CAN_MONITOR = jack_sys::JackPortCanMonitor,

        /// PortIsTerminal means:
        ///
        /// * for an input port: the data received by the port will not be passed on or made
        /// available at any other port
        ///
        /// * for an output port: the data available at the port does not originate from any other
        /// port
        ///
        /// * Audio synthesizers, I/O hardware interface clients, HDR systems are examples of
        ///   clients that would set this flag for their ports.
        const PORT_IS_TERMINAL = jack_sys::JackPortIsTerminal,
    }
}
}

/// This module contains default port type constants
pub mod port_type {
    // these are #defines in the jack source so jack_sys doesn't pick them up
    pub const DEFAULT_AUDIO_TYPE: &'static str = "32 bit float mono audio";
    pub const DEFAULT_MIDI_TYPE: &'static str = "8 bit raw midi";
}

/// This module contains a bitflags! generated struct for jack error codes, and some constants
/// defining their default values
pub mod status {
use jack_sys;

bitflags! {
    pub flags Status: u32 {
        /// overall operation failed
        const FAILURE = jack_sys::JackFailure,

        /// The operation contained an invalid or unsupported option.
        const INVALID_OPTION = jack_sys::JackInvalidOption,

        /// The desired client name was not unique.  With the JackUseExactName option this
        /// situation is fatal.  Otherwise, the name was modified by appending a dash and a
        /// two-digit number in the range "-01" to "-99".  If the specified client name plus these
        /// extra characters would be too long, the open fails instead.
        const NAME_NOT_UNIQUE = jack_sys::JackNameNotUnique,

        /// The JACK server was started as a result of this operation.  Otherwise, it was running
        /// already.  In either case the caller is now connected to jackd, so there is no race
        /// condition.  When the server shuts down, the client will find out.
        const SERVER_STARTED = jack_sys::JackServerStarted,

        /// Unable to connect to the JACK server.
        const SERVER_FAILED = jack_sys::JackServerFailed,

        /// Communication error with the JACK server.
        const SERVER_ERROR = jack_sys::JackServerError,

        /// Requested client does not exist.
        const NO_SUCH_CLIENT = jack_sys::JackNoSuchClient,

        /// Unable to load internal client
        const LOAD_FAILURE = jack_sys::JackLoadFailure,

        /// Unable to initialize client
        const INIT_FAILURE = jack_sys::JackInitFailure,

        /// Unable to access shared memory
        const SHM_FAILURE = jack_sys::JackShmFailure,

        /// Client's protocol version does not match
        const VERSION_ERROR = jack_sys::JackVersionError,

        /// Backend error
        const BACKEND_ERROR = jack_sys::JackBackendError,

        /// Client zombified failure
        const CLIENT_ZOMBIE = jack_sys::JackClientZombie
    }
}
}

pub mod options {
use jack_sys;

bitflags! {
    pub flags Options: u32 {
        /// Null value to use when no option bits are needed.
        const NULL_OPTIONS = jack_sys::JackNullOption,

        /// Do not automatically start the JACK server when it is not already running.  This option
        /// is always selected if $JACK_NO_START_SERVER is defined in the calling process
        /// environment.
        const NO_START_SERVER = jack_sys::JackNoStartServer,

        /// Use the exact client name requested.  Otherwise, JACK automatically generates a unique
        /// one, if needed.
        const USE_EXACT_NAME = jack_sys::JackUseExactName,

        // TODO these flags are disabled because the client interface doesn't support them
        // const SERVER_NAME = jack_sys::JackServerName,
        // const LOAD_NAME = jack_sys::JackLoadName,
        // const LOAD_INIT = jack_sys::JackLoadInit,
        // const SESSION_ID = jack_sys::JackSessionID, // TODO what is this?
    }
}
}
