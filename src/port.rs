use jack_sys;
use std::slice;
use std::ffi::CStr;


/// A port handle can be retrieved from the client for any port on the server, including the ports
/// the client owns
/// Using a PortHandle isn't entirely safe. A PortHandle may continue to exist after a port has
/// been deregistered. Operating on a PortHandle which has been deregistered is undefined behavior
#[derive(Debug, Clone, Copy)]
pub struct PortHandle {
    c_port: *mut jack_sys::jack_port_t
}

/// All JACK operations on ports are thread safe
unsafe impl Sync for PortHandle { }

impl PortHandle {
    /// Creates a new port handle from C JACK library pointer.
    /// Should not be used outside of the `easyjack` code.
    #[doc(hidden)]
    pub fn new(c_port: *mut jack_sys::jack_port_t) -> Self {
        PortHandle { c_port: c_port }
    }

    /// Gets the port's assigned full name (including the client name and the colon)
    pub fn get_name(&self) -> &str {
        unsafe {
            let raw = self.c_port;
            let cstr = jack_sys::jack_port_name(raw);
            CStr::from_ptr(cstr).to_str().unwrap()
        }
    }

    // TODO get_short_name

    /// Get a readable buffer for the port. This is valid for any type of port, but probably
    /// doesn't make sense for an output port.
    ///
    /// This is for use in the `process` callback
    pub fn get_read_buffer(&self, nframes: jack_sys::jack_nframes_t)
        -> &[jack_sys::jack_default_audio_sample_t]
    {
        self.get_write_buffer(nframes)
    }

    /// Get a writable buffer for the port. This is only valid for an output port.
    /// If the port is not an output port this will `panic!`
    ///
    /// TODO maybe be more clever with the type system to prevent calls on output ports
    ///
    /// This is for use in the `process` callback
    pub fn get_write_buffer(&self, nframes: jack_sys::jack_nframes_t)
        -> &mut [jack_sys::jack_default_audio_sample_t]
    {
        unsafe {
            let ptr = jack_sys::jack_port_get_buffer(self.c_port, nframes);
            let ptr = ptr as *mut jack_sys::jack_default_audio_sample_t;
            slice::from_raw_parts_mut(ptr, nframes as usize)
        }
    }

    /// Gets the raw C JACK pointer
    /// Not to be used outside of the `easyjack` code
    #[doc(hidden)]
    pub unsafe fn get_raw(&self) -> *mut jack_sys::jack_port_t {
        self.c_port
    }
}
