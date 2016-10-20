extern crate jack_sys;
use types::*;

/// This module defines a trait for each of the possible callbacks which may be
/// implemented for interaction with the jack API
/// Note that these callback handlers do not have thread safety marker
/// constraints because the client always takes ownership of the callback
/// handlers, ensuring that the callbacks will only be called in a thread safe
/// manner
/// TODO verify that this makes sense

pub struct CallbackContext { }

impl CallbackContext {
    pub fn new() -> Self {
        CallbackContext { }
    }
}

/// This trait defines a handler for the process callback
pub trait ProcessHandler {
    fn process(&mut self, ctx: &CallbackContext, nframes: NumFrames) -> i32;
}

/// Struct which handles all metadata operations
pub trait MetadataHandler {
    #[allow(unused_variables)]
    fn sample_rate_changed(&mut self, srate: NumFrames) -> i32 { 0 }

    #[allow(unused_variables)]
    fn on_port_connect(&mut self, a: PortId, b: PortId, status: PortConnectStatus) { }

    fn callbacks_of_interest(&self) -> Vec<MetadataHandlers>;
}

pub enum MetadataHandlers {
    SampleRate,
    PortConnect,
    Shutdown,
    Freewheel,
    BufferSize,
    ClientRegistration,
    PortRegistration,
    PortRename,
    GraphOrder,
    Xrun,
}
