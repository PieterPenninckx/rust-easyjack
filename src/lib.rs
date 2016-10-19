#[macro_use]
extern crate bitflags;
extern crate jack_sys;
extern crate libc;
extern crate num;

// all the modules
mod client;
mod callbackhandler;
mod port;
mod types;
mod midi;

// get everything into this namespace
pub use callbackhandler::*;
pub use client::*;
pub use midi::*;
pub use port::*;
pub use types::*;
