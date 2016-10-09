use jack_sys;

use std::ffi::CStr;
use std::marker::PhantomData;
use std::slice;
use std::str::FromStr;

use types::*;

type Jackptr = *mut jack_sys::jack_port_t;

/// Ports are the means by which jack clients communicate with each other.
///
/// The port wrappers in `easyjack` have slightly confusing type definitions due to the behavior of
/// the underlying JACK C API
///
/// Each of the Port structs defined implement the `Port` trait, which should be sufficient for use
/// in many situations, however, occasionally, we may want to access pieces of data which are only
/// available on specific types of ports.
///
/// Because the JACK C api handles all ports with a jack_port_t structure, we are limited in our
/// ability to determine the exact properties of a port at compile time. For example, if you ask
/// the JACK API to give you a port, by name, you cannot know all of the various properties of the
/// port without additional inspection. This additional inspection will incur additional costs, so
/// we have chosen to make all additional inspection optional.
///
/// This means that many of the `easyjack` API methods will return an `UnknownPortHandle`, and
/// users of the API methods will have the option to attempt to promote this `UnknownPortHandle` to
/// ports of different types. These attempts at conversion will perform additional inspection of
/// the port's flags (unless the unsafe versions are used)
///
/// One additional note about Port types.
/// All of these port types are only handles to underlying ports (think of them as an index into a
/// vector).
/// All of these port types implement `Copy`.
/// This means that a Port "handle" may become invalid if the port becomes invalid.
/// Using a port after it has become invalid is undefined behavior and may cause all sorts of
/// strange things to occur.
pub trait Port {
    /// Gets the port's assigned full name (including the client name and the colon)
    fn get_name(&self) -> String {
        unsafe {
            let raw = self.get_raw();
            let cstr = jack_sys::jack_port_name(raw);

            // do a little dance to alleviate ownership pains
            String::from_str(CStr::from_ptr(cstr).to_str().unwrap()).unwrap()
        }
    }

    // TODO many other functions

    /// Get the flags used to construct this port
    fn get_port_flags(&self) -> port_flags::PortFlags {
        let rawbits = unsafe { jack_sys::jack_port_flags(self.get_raw()) };
        port_flags::PortFlags::from_bits(rawbits as u32).unwrap()
    }

    /// Gets the raw C JACK pointer
    /// Not to be used outside of the `easyjack` code
    #[doc(hidden)]
    unsafe fn get_raw(&self) -> Jackptr;
}

pub struct UnknownPortHandle {
    c_port: Jackptr
}

impl UnknownPortHandle {
    #[doc(hidden)]
    pub fn new(c_port: Jackptr) -> Self {
        UnknownPortHandle { c_port: c_port }
    }

    /// Attempts to coerce the port into an input port
    /// This function will test the port's flags to ensure that it is actually an input port
    pub fn as_input<SampleType>(self) -> Option<InputPortHandle<SampleType>> {
        let flags = self.get_port_flags();
        if flags.contains(port_flags::PORT_IS_INPUT) {
            Some(InputPortHandle::<SampleType>::new(self.c_port))
        } else {
            None
        }
    }

    /// Attempts to coerce the port into an output port
    /// This function will test the port's flags to ensure that it is actually an output port
    pub fn as_output<SampleType>(self) -> Option<OutputPortHandle<SampleType>> {
        let flags = self.get_port_flags();
        if flags.contains(port_flags::PORT_IS_OUTPUT) {
            Some(OutputPortHandle::<SampleType>::new(self.c_port))
        } else {
            None
        }
    }

    /// Forces coercion to an input port
    /// This is marked unsafe because it DOES NOT check the port flags before coercing it to the
    /// new type.
    /// If you are 100% sure your port is an input port, this call can save you some extra
    /// operations. If not, use the safe version!
    pub unsafe fn force_as_input<SampleType>(self) -> InputPortHandle<SampleType> {
        InputPortHandle::<SampleType>::new(self.c_port)
    }

    /// Forces coercion to an output port
    /// This is marked unsafe because it DOES NOT check the port flags before coercing it to the
    /// new type.
    /// If you are 100% sure your port is an output port, this call can save you some extra
    /// operations. If not, use the safe version!
    pub unsafe fn force_as_output<SampleType>(self) -> OutputPortHandle<SampleType> {
        OutputPortHandle::<SampleType>::new(self.c_port)
    }
}

impl Port for UnknownPortHandle {
    #[doc(hidden)]
    unsafe fn get_raw(&self) -> Jackptr { self.c_port }
}

#[derive(Debug, Clone, Copy)]
pub struct InputPortHandle<SampleType> {
    c_port: Jackptr,
    phantom: PhantomData<SampleType>
}

impl<SampleType> Port for InputPortHandle<SampleType> {
    #[doc(hidden)]
    unsafe fn get_raw(&self) -> Jackptr { self.c_port }
}

impl<SampleType> InputPortHandle<SampleType> {
    #[doc(hidden)]
    pub fn new(c_port: Jackptr) -> Self {
        InputPortHandle {
            c_port: c_port,
            phantom: PhantomData,
        }
    }

    /// Get the input port's readable buffer
    pub fn get_read_buffer(
        &self,
        nframes: NumFrames)
        -> &[SampleType]
    {
        unsafe {
            let ptr = jack_sys::jack_port_get_buffer(self.c_port, nframes);
            let ptr = ptr as *mut SampleType;
            slice::from_raw_parts_mut(ptr, nframes as usize)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OutputPortHandle<SampleType> {
    c_port: Jackptr,
    phantom: PhantomData<SampleType>
}

impl<SampleType> Port for OutputPortHandle<SampleType> {
    #[doc(hidden)]
    unsafe fn get_raw(&self) -> Jackptr { self.c_port }
}

impl<SampleType> OutputPortHandle<SampleType> {
    #[doc(hidden)]
    pub fn new(c_port: Jackptr) -> Self {
        OutputPortHandle {
            c_port: c_port,
            phantom: PhantomData,
        }
    }

    /// Get the input port's readable buffer
    pub fn get_write_buffer(
        &self,
        nframes: NumFrames)
        -> &mut [SampleType]
    {
        unsafe {
            let ptr = jack_sys::jack_port_get_buffer(self.c_port, nframes);
            let ptr = ptr as *mut SampleType;
            slice::from_raw_parts_mut(ptr, nframes as usize)
        }
    }
}

// TODO some nice type aliases to hide all this magic and craziness
