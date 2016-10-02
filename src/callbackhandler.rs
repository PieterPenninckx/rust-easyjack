extern crate jack_sys;
use types::*;

/// This module defines a trait for each of the possible callbacks which may be
/// implemented for interaction with the jack API
/// Note that these callback handlers do not have thread safety marker
/// constraints because the client always takes ownership of the callback
/// handlers, ensuring that the callbacks will only be called in a thread safe
/// manner
/// TODO verify that this makes sense

/// This trait defines a handler for the process callback
pub trait ProcessHandler {
    fn process(&mut self, nframes: NumFrames) -> i32;
}

pub trait ShutdownHandler { }
pub trait FreewheelHandler { }
pub trait BufferSizeHandler { }
pub trait SampleRateHandler { }
pub trait ClientRegistrationHandler { }
pub trait PortRegistrationHandler { }

pub trait PortConnectHandler {
    fn on_connect(&mut self, a: PortId, b: PortId, status: PortConnectStatus);
}

pub trait PortRenameHandler { }
pub trait GraphOrderHandler { }
pub trait XrunHandler { }
