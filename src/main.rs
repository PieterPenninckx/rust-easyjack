extern crate chan_signal;
extern crate jack_sys;
extern crate libc;

use std::ptr::{null_mut};
use std::ffi::{CString, CStr};
use std::slice;

use chan_signal::Signal;

trait CallbackHandler: Sync {
    fn process(&mut self, nframes: jack_sys::jack_nframes_t) -> i32;
}

#[derive(Copy, Clone, Debug)]
enum PortType {
    DefaultAudioType,
    DefaultMidiType
}

impl PortType {
    pub fn as_string(self) -> &'static str {
        match self {
            PortType::DefaultAudioType => "32 bit float mono audio",
            PortType::DefaultMidiType  => "8 bit raw midi"
        }
    }
}

/// client disconnects from the server when it goes out of scope
/// the client owns a number of ports
struct Client<T: CallbackHandler> {
    c_client: *mut jack_sys::jack_client_t,
    handler: Option<T>
}

impl<T: CallbackHandler> Client<T> {
    pub fn open(name: &str) -> Result<Self, &str> {
        let cstr = CString::new(name).unwrap();

        let cl = unsafe {
            jack_sys::jack_client_open(cstr.as_ptr(), 0, null_mut::<jack_sys::jack_status_t>())
        };

        if cl.is_null() {
            Err("TODO return a better error code. Basically something went wrong sorry")
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

    pub fn register_port(&mut self, name: &str, ptype: PortType, opts: u64)
        -> Result<PortHandle, &str>
    {
        // TODO does this need to live longer?
        let cstr = CString::new(name).unwrap();
        let typestr = CString::new(ptype.as_string()).unwrap();

        let port = unsafe {
            jack_sys::jack_port_register(
                self.c_client,
                cstr.as_ptr(),
                typestr.as_ptr(),
                opts as u64,
                0)
        };

        if port.is_null() {
            Err("TODO return a better error you loser")
        } else {
            let port = PortHandle { c_port: port };
            println!("porthandle={:?}", port);
            Ok(port)
        }
    }

    pub fn unregister_port(&mut self, port: PortHandle) -> Result<(), &str> {
        let ret = unsafe {
            jack_sys::jack_port_unregister(self.c_client, port.get_raw())
        };

        if ret == 0 {
            Ok(())
        } else {
            Err("TODO errors wah")
        }
    }

    pub fn set_handler(&mut self, handler: T) {
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

        unsafe {
            let mut handler = self.handler.as_mut().unwrap();
            let ptr = handler as *mut _ as *mut libc::c_void;
            jack_sys::jack_set_process_callback(self.c_client, Some(process_callback::<T>), ptr);
        }
    }

    /// tells the jack server that the client is read to start processing audio
    /// This will initiate callbacks into the jack callback function provided. Each callback will
    /// be executed in a different thread
    /// This thread will be setup by jack
    pub fn activate(&self) -> Result<(), &str> {
        // TODO disable various other function calls after activate is called
        // do this via (self) -> ActivatedClient or something
        unsafe {
            jack_sys::jack_activate(self.c_client);
        }

        Ok(())
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

/// A port handle can be retrieved from the client for any port on the server, including the ports
/// the client owns
/// Using a PortHandle isn't entirely safe. A PortHandle may continue to exist after a port has
/// been deregistered. Operating on a PortHandle which has been deregistered is undefined behavior
#[derive(Debug, Clone, Copy)]
struct PortHandle {
    c_port: *mut jack_sys::jack_port_t
}

// jack promises that operations on ports are thread safe
unsafe impl Sync for PortHandle { }

impl PortHandle {
    pub fn get_name(&self) -> &str {
        unsafe {
            let raw = self.c_port;
            let cstr = jack_sys::jack_port_name(raw);
            CStr::from_ptr(cstr).to_str().unwrap()
        }
    }

    pub fn get_read_buffer(&self, nframes: jack_sys::jack_nframes_t)
        -> &[jack_sys::jack_default_audio_sample_t]
    {
        self.get_write_buffer(nframes)
    }

    pub fn get_write_buffer(&self, nframes: jack_sys::jack_nframes_t)
        -> &mut [jack_sys::jack_default_audio_sample_t]
    {
        unsafe {
            let ptr = jack_sys::jack_port_get_buffer(self.c_port, nframes);
            let ptr = ptr as *mut jack_sys::jack_default_audio_sample_t;
            slice::from_raw_parts_mut(ptr, nframes as usize)
        }
    }

    pub unsafe fn get_raw(&self) -> *mut jack_sys::jack_port_t {
        self.c_port
    }
}

struct Connector {
    inputs: Vec<PortHandle>,
    outputs: Vec<PortHandle>
}

impl Connector {
    pub fn new(inputs: Vec<PortHandle>, outputs: Vec<PortHandle>) -> Self {
        assert_eq!(inputs.len(), outputs.len());

        Connector {
            inputs: inputs,
            outputs: outputs
        }
    }
}

impl CallbackHandler for Connector {
    fn process(&mut self, nframes: jack_sys::jack_nframes_t) -> i32 {
        // for each of our inputs and outputs, copy the input buffer into the output buffer
        for index in 0..self.inputs.len() {
            let i = self.inputs[index].get_read_buffer(nframes);
            let o = self.outputs[index].get_write_buffer(nframes);
            o.clone_from_slice(i);
        }

        // return 0 so jack lets us keep running
        0
    }
}

fn main() {
    // set up signal handlers using chan_signal
    let signal = chan_signal::notify(&[Signal::INT]);

    let mut jack_client = Client::open("testclient").unwrap();
    println!("client created named: {}", jack_client.get_name());

    // 2 in, 2 out
    let input1 =
        jack_client.register_port("input1", PortType::DefaultAudioType, 0x1).unwrap();

    let input2 =
        jack_client.register_port("input2", PortType::DefaultAudioType, 0x1).unwrap();

    let output1 =
        jack_client.register_port("output1", PortType::DefaultAudioType, 0x2).unwrap();

    let output2 =
        jack_client.register_port("output2", PortType::DefaultAudioType, 0x2).unwrap();

    let handler = Connector::new(vec![input1, input2], vec![output1, output2]);
    jack_client.set_handler(handler);

    // start everything up
    jack_client.activate().unwrap();

    // wait to get a SIGINT
    // jack will do all of its magic in other threads
    signal.recv().unwrap();

    // now we can clean everything up
    // the library doesn't handle this for us because it would be rather confusing, especially
    // given how the underlying jack api actually works

    // unregister all the ports
    jack_client.close().unwrap();

    for p in vec![input1, input2, output1, output2] {
        jack_client.unregister_port(p).unwrap();
    }

    // exit gracefully
}
