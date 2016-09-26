extern crate jack_sys;

pub trait CallbackHandler: Sync {
    fn process(&mut self, nframes: jack_sys::jack_nframes_t) -> i32;
}

