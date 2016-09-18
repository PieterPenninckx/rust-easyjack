extern crate jack_sys;
extern crate libc;

use jack_sys::*;

use std::ptr::{null, null_mut};
use std::mem::transmute;

use std::ffi::{CString, CStr};
use std::{thread, time};

enum PortType {
    DefaultAudioType,
    DefaultMidiType
}

/// client disconnects from the server when it goes out of scope
/// the client owns a number of ports
struct Client {
    c_client: *mut jack_client_t,
    owned_ports: Vec<PrivatePort>
}

impl Client {
    pub fn open(name: &str) -> Result<Self, &str> {
        let cstr = CString::new(name).unwrap();
        let cl = unsafe { jack_client_open(cstr.as_ptr(), 0, null_mut::<jack_status_t>())};
        if cl.is_null() {
            // TODO return a better error code
            Err("something bad happened")
        } else {
            Ok(Client {
                c_client: cl,
                owned_ports: Vec::new()
            })
        }
    }

    pub fn get_name(&self) -> &str {
        unsafe {
            let raw = self.c_client;
            let cstr = jack_get_client_name(raw);
            CStr::from_ptr(cstr).to_str().unwrap()
        }

        // use jack's getters and setters because the names are subject to change
        // do not need to free the string
    }

    pub fn create_port(&mut self, name: &str, ptype: PortType) -> Port {
        // TODO does this need to live longer?
        let cstr = CString::new(name).unwrap();

        // TODO perhaps be somewhat more clever, if needed
        let typestr = match ptype {
            PortType::DefaultAudioType => "32 bit float mono audio",
            PortType::DefaultMidiType  => "8 bit raw midi"
        };
        let typestr = CString::new(typestr).unwrap();

        let port = unsafe {
            jack_port_register(
                self.c_client,
                cstr.as_ptr(),
                typestr.as_ptr(),
                JackPortIsInput as u64,
                0)
        };

        let port = PrivatePort { c_client: self.c_client, c_port: port };
        self.owned_ports.push(port);

        Port { p: self.owned_ports.last_mut().unwrap() }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe {
            // TODO check error code
            jack_client_close(self.c_client);
        }
    }
}

/// the only way to create a port is via a client
/// when a port goes out of scope, it gets deregistered
/// TODO make sure that a port cannot live longer than it's client
struct PrivatePort {
    // keep a copy of the client that owns this port
    c_client: *mut jack_client_t,
    c_port: *mut jack_port_t
}

impl Drop for PrivatePort {
    fn drop(&mut self) {
        unsafe {
            jack_port_unregister(self.c_client, self.c_port);
        }
    }
}

struct Port<'a> {
    p: &'a PrivatePort
}

fn main() {
    let mut jack_client = Client::open("testclient").unwrap();
    println!("client created named: {}", jack_client.get_name());

    {
        let input  = jack_client.create_port("input", PortType::DefaultMidiType);
        let output = jack_client.create_port("output", PortType::DefaultAudioType);

        let mut i = 0;
        while i < 100 {
            i += 1;
            let ten_millis = time::Duration::from_millis(10);
            let now = time::Instant::now();

            thread::sleep(ten_millis);
        }
    }

    println!("next");

    let mut i = 0;
    while i < 100 {
        i += 1;
        let ten_millis = time::Duration::from_millis(10);
        let now = time::Instant::now();

        thread::sleep(ten_millis);
    }
}
