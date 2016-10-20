use jack_sys;
use libc;
use std::marker::PhantomData;
use std::mem;
use std::slice;
use types::*;

pub struct MidiEventBuf<'a> {
    num: usize,
    all_events_buffer: *mut libc::c_void,

    // only exists to enforce the lifetime
    phantom: PhantomData<&'a libc::c_void>
}

impl<'a> MidiEventBuf<'a> {
    #[doc(hidden)]
    pub unsafe fn new(jackptr: *mut libc::c_void) -> Self {
        assert!(!jackptr.is_null());

        MidiEventBuf {
            num: jack_sys::jack_midi_get_event_count(jackptr) as usize,
            all_events_buffer: jackptr,
            phantom: PhantomData
        }
    }

    /// This looks like it isn't a reference, but it is. Trust me.
    pub fn get(&self, index: usize) -> MidiEventRef {
        assert!(!self.all_events_buffer.is_null());

        if index >= self.num {
            panic!("index out of bounds");
        }

        unsafe {
            let mut jstruct = mem::uninitialized();
            let ret = jack_sys::jack_midi_event_get(
                &mut jstruct,
                self.all_events_buffer,
                index as u32);

            if ret != 0 {
                panic!("index out of bounds/ENODATA");
            }

            MidiEventRef::new(jstruct)
        }
    }

    pub fn len(&self) -> usize { self.num }
}

/// A structure representing a midi event
pub struct MidiEvent { }

/// A reference to a midi event contained in a MidiEventBuf
/// These references do actually perform some logic, so a plain &MidiEvent would not be sufficient
/// for our binding needs
#[derive(Debug)]
pub struct MidiEventRef<'a> {
    time: NumFrames,
    len: libc::size_t,
    buffer: *mut jack_sys::jack_midi_data_t,

    // only exists to enforce the lifetime
    phantom: PhantomData<&'a jack_sys::jack_midi_data_t>
}

impl<'a> MidiEventRef<'a> {
    #[doc(hidden)]
    pub unsafe fn new(jackstruct: jack_sys::Struct__jack_midi_event) -> Self {
        assert!(!jackstruct.buffer.is_null());

        // its easier to access everything if we pull all of the data out of the struct and store
        // it ourselves
        MidiEventRef {
            time:    jackstruct.time,
            len:     jackstruct.size,
            buffer:  jackstruct.buffer,
            phantom: PhantomData,
        }
    }

    /// Returns the raw midi data corresponding to this event
    pub fn raw_midi_bytes(&self) -> &[u8] {
        // the ptr cannot be null, else this entire thing is malformed
        assert!(!self.buffer.is_null());

        unsafe {
            slice::from_raw_parts(self.buffer, self.len)
        }
    }

    pub fn get_jack_time(&self) -> NumFrames {
        assert!(!self.buffer.is_null());
        self.time
    }
}
