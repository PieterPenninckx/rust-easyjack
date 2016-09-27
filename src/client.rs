use jack_sys;
use libc;

use std::ffi::{CString, CStr};

use callbackhandler::CallbackHandler;
use port::*;
use types::*;

/// A jack client connected to a jack server
/// TODO example
pub struct Client<T: CallbackHandler> {
    c_client: *mut jack_sys::jack_client_t,
    handler: Option<T>
}

impl<T: CallbackHandler> Client<T> {
    /// Creates a new client and connects it to the default jack server.
    /// The client will use the name given. If the name is not unique, the behavior depends on the
    /// options provided via `opts`
    ///
    /// TODO this interface is not entirely correct, fix it. There is potential for a status to be
    /// returned even if the creation fails
    ///
    /// TODO client_name_size details in docs and in code
    pub fn open(name: &str, opts: options::Options) -> Result<Self, status::Status> {
        // TODO does jack check if the options are valid?
        let cstr = CString::new(name).unwrap();
        let mut status = 0 as jack_sys::jack_status_t;
        let statusptr = &mut status as *mut jack_sys::jack_status_t;

        let cl = unsafe {
            jack_sys::jack_client_open(cstr.as_ptr(), opts.bits(), statusptr)
        };

        if cl.is_null() {
            // if this fails, we are accepting a potential panic
            let status = status::Status::from_bits(status).unwrap();
            Err(status)
        } else {
            // TODO check status anyway
            Ok(Client {
                c_client: cl,
                handler: None
            })
        }
    }

    /// Returns the actual name of the client. This is useful when USE_EXACT_NAME is not specified,
    /// because the jack server might assign some other name to your client to ensure that it is
    /// unique.
    pub fn get_name(&self) -> &str {
        // use jack's getters and setters because the names are subject to change
        // do not need to free the string
        unsafe {
            let raw = self.c_client;
            let cstr = jack_sys::jack_get_client_name(raw);
            CStr::from_ptr(cstr).to_str().unwrap()
        }
    }

    /// Create a new port for this client. Ports are used to move data in and out of the client
    /// (audio data, midi data, etc). Ports may be connected to other ports in various ways.
    ///
    /// Each port has a short name which must be unique among all the ports owned by the client.
    /// The port's full name contains the name of the client, followed by a colon (:), followed by
    /// the port's short name.
    ///
    /// All ports have a type. The `port_type` module contains port types which may be used.
    ///
    /// You may also specify a number of flags from the `port_flags` module which control the
    /// behavior of the created port (input vs output, etc)
    ///
    /// TODO something about buffer size I haven't figured out yet
    /// TODO port_name_size()
    pub fn register_port(&mut self, name: &str, ptype: PortType, opts: port_flags::PortFlags)
        -> Result<PortHandle, status::Status>
    {
        let cstr = CString::new(name).unwrap();
        let typestr = CString::new(ptype).unwrap();

        let port = unsafe {
            jack_sys::jack_port_register(
                self.c_client,
                cstr.as_ptr(),
                typestr.as_ptr(),
                opts.bits() as u64,
                0)
        };

        if port.is_null() {
            // no error code is returned from jack here
            Err(status::FAILURE)
        } else {
            let port = PortHandle::new(port);
            Ok(port)
        }
    }

    /// Helper function which registers an input audio port with a given name.
    pub fn register_input_audio_port(&mut self, name: &str) -> Result<PortHandle, status::Status> {
        self.register_port(name, port_type::DEFAULT_AUDIO_TYPE, port_flags::PORT_IS_INPUT)
    }

    /// Helper function which registers an output audio port with a given name.
    pub fn register_output_audio_port(&mut self, name: &str) -> Result<PortHandle, status::Status> {
        self.register_port(name, port_type::DEFAULT_AUDIO_TYPE, port_flags::PORT_IS_OUTPUT)
    }

    /// Removes the port from the client and invalidates the port and all PortHandles referencing
    /// the port.
    ///
    /// The server disconnects everything that was previously connected to the port.
    pub fn unregister_port(&mut self, port: PortHandle) -> Result<(), status::Status> {
        let ret = unsafe {
            jack_sys::jack_port_unregister(self.c_client, port.get_raw())
        };

        if ret == 0 {
            Ok(())
        } else {
            // TODO try to handle this error code
            Err(status::FAILURE)
        }
    }

    /// Set the client's callback handler.
    /// See the docs for the `CallbackHandler` struct for more details
    pub fn set_handler(&mut self, handler: T) -> Result<(), status::Status>{
        // a function which will do some setup then call the client's handler
        // this is called by a separate thread which rust is entirely aware of, so be careful
        extern "C" fn process_callback<T: CallbackHandler>(
            nframes: jack_sys::jack_nframes_t,
            args: *mut libc::c_void) -> libc::c_int
        {
            let this = args as *mut T;
            unsafe { (*this).process(nframes) }
        }

        // setting the handler here moves it into it's permanent location
        self.handler = Some(handler);

        let ret = unsafe {
            let mut handler = self.handler.as_mut().unwrap();
            let ptr = handler as *mut _ as *mut libc::c_void;
            jack_sys::jack_set_process_callback(self.c_client, Some(process_callback::<T>), ptr)
        };

        if ret != 0 {
            // again, no error code provided
            Err(status::FAILURE)
        } else {
            Ok(())
        }
    }

    /// tells the JACK server that the client is read to start processing audio This will initiate
    /// callbacks into the `CallbackHandler` provided.
    pub fn activate(&self) -> Result<(), status::Status> {
        // TODO disable various other function calls after activate is called
        // do this via (self) -> ActivatedClient or something
        let ret = unsafe {
            jack_sys::jack_activate(self.c_client)
        };

        if ret != 0 {
            // TODO handle error
            Err(status::FAILURE)
        } else {
            Ok(())
        }
    }

    /// Disconnects the client from the JACK server.
    /// This will also disconnect and destroy any of the ports which the client registered
    pub fn close(&mut self) -> Result<(), &str> {
        let ret = unsafe {
            jack_sys::jack_client_close(self.c_client)
        };

        if ret == 0 {
            Ok(())
        } else {
            Err("some error should go here")
        }
    }
}
