use jack_sys;
use libc;

use std::ffi::{CString, CStr};

use callbackhandler::CallbackHandler;
use port::*;
use types::*;

pub struct Client<T: CallbackHandler> {
    c_client: *mut jack_sys::jack_client_t,
    handler: Option<T>
}

impl<T: CallbackHandler> Client<T> {
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
            Ok(Client {
                c_client: cl,
                handler: None
            })
        }
    }

    pub fn get_name(&self) -> &str {
        // use jack's getters and setters because the names are subject to change
        // do not need to free the string
        unsafe {
            let raw = self.c_client;
            let cstr = jack_sys::jack_get_client_name(raw);
            CStr::from_ptr(cstr).to_str().unwrap()
        }
    }

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

    pub fn register_input_audio_port(&mut self, name: &str) -> Result<PortHandle, status::Status> {
        self.register_port(name, port_type::DEFAULT_AUDIO_TYPE, port_flags::PORT_IS_INPUT)
    }

    pub fn register_output_audio_port(&mut self, name: &str) -> Result<PortHandle, status::Status> {
        self.register_port(name, port_type::DEFAULT_AUDIO_TYPE, port_flags::PORT_IS_OUTPUT)
    }

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

    /// tells the jack server that the client is read to start processing audio
    /// This will initiate callbacks into the jack callback function provided. Each callback will
    /// be executed in a different thread
    /// This thread will be setup by jack
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
