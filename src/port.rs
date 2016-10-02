use jack_sys;

use std::ffi::CStr;
use std::slice;
use std::str::FromStr;

use types::*;

/// A port handle can be retrieved from the client for any port on the server, including the ports
/// the client owns
/// Using a PortHandle isn't entirely safe. A PortHandle may continue to exist after a port has
/// been deregistered. Operating on a PortHandle which has been deregistered is undefined behavior
#[derive(Debug, Clone, Copy)]
pub struct PortHandle {
    c_port: *mut jack_sys::jack_port_t,
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
    pub fn get_name(&self) -> String {
        unsafe {
            let raw = self.c_port;
            let cstr = jack_sys::jack_port_name(raw);

            // do a little dance to alleviate ownership pains
            String::from_str(CStr::from_ptr(cstr).to_str().unwrap()).unwrap()
        }
    }

    // TODO get_short_name

    // TODO get other types of buffers (currently only supports audio buffers)

    /// Get a readable buffer for the port. This is valid for any type of port, but probably
    /// doesn't make sense for an output port.
    ///
    /// This is for use in the `process` callback
    pub fn get_read_buffer(
        &self,
        nframes: NumFrames)
        -> &[DefaultAudioSample]
    {
        self.get_write_buffer(nframes)
    }

    /// Get a writable buffer for the port. This is only valid for an output port.
    /// If the port is not an output port this will `panic!`
    ///
    /// TODO maybe be more clever with the type system to prevent calls on output ports
    ///
    /// This is for use in the `process` callback
    pub fn get_write_buffer(
        &self,
        nframes: NumFrames)
        -> &mut [DefaultAudioSample]
    {
        // TODO actually panic :(
        unsafe {
            let ptr = jack_sys::jack_port_get_buffer(self.c_port, nframes);
            let ptr = ptr as *mut jack_sys::jack_default_audio_sample_t;
            slice::from_raw_parts_mut(ptr, nframes as usize)
        }
    }

    /// Get the flags used to construct this port
    pub fn get_port_flags(&self) -> port_flags::PortFlags {
        let rawbits = unsafe { jack_sys::jack_port_flags(self.c_port) };

        port_flags::PortFlags::from_bits(rawbits as u32).unwrap()
    }

    /// Gets the raw C JACK pointer
    /// Not to be used outside of the `easyjack` code
    #[doc(hidden)]
    pub unsafe fn get_raw(&self) -> *mut jack_sys::jack_port_t {
        self.c_port
    }
}
