#[macro_use]
extern crate bitflags;
extern crate jack_sys;
extern crate libc;

// all the modules
mod client;
mod callbackhandler;
mod port;
mod types;

// get everything into this namespace
pub use client::*;
pub use callbackhandler::*;
pub use port::*;
pub use types::*;
