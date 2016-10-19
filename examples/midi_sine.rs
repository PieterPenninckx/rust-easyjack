// use the jack bindings
extern crate easyjack as jack;

// use nix for signal handling
// see simple_client example for some description of how this works
extern crate nix;

use nix::sys::signal;
use std::f32::consts;
use std::f32;
use std::sync::atomic;
use std::sync::mpsc::{SyncSender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

static RUNNING: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

type IPort = jack::InputPortHandle<jack::MidiEvent>;
type OPort = jack::OutputPortHandle<jack::DefaultAudioSample>;

/// struct to handle metadata operations
struct MetadataHandler {
    outgoing: SyncSender<[jack::DefaultAudioSample; 128]>,
}

impl MetadataHandler {
    pub fn new(outgoing: SyncSender<[jack::DefaultAudioSample; 128]>) -> Self {
        MetadataHandler { outgoing: outgoing }
    }
}

impl jack::SampleRateHandler for MetadataHandler {
    fn sample_rate_changed(&mut self, srate: jack::NumFrames) -> i32 {
        println!("updating sample rate: {}", srate);

        let f = AudioHandler::calc_note_freqs(srate);
        match self.outgoing.send(f) {
            Ok(_) => 0,
            Err(_) => 0
        }
    }
}

/// struct to handle audio event loop
struct AudioHandler {
    input:  jack::InputPortHandle<jack::MidiEvent>,
    output: jack::OutputPortHandle<jack::DefaultAudioSample>,

    // TODO explain what this is all about
    ramp:       jack::DefaultAudioSample,
    note_on:    jack::DefaultAudioSample,
    note:       u8,
    note_freqs: [jack::DefaultAudioSample; 128],

    incoming: Receiver<[jack::DefaultAudioSample; 128]>,
}

impl AudioHandler {
    pub fn new(input: IPort, output: OPort, incoming: Receiver<[jack::DefaultAudioSample; 128]>)
        -> AudioHandler
    {
        let freqs = [0.0; 128];
        AudioHandler {
            input:      input,
            output:     output,
            ramp:       0.0,
            note_on:    0.0,
            note:       0,
            note_freqs: freqs,
            incoming:   incoming,
        }
    }

    pub fn calc_note_freqs(srate: jack::NumFrames) -> [jack::DefaultAudioSample; 128] {
        println!("recalculating note frequencies");
        let mut freqs = [0.0; 128];
        print!("new_freqs: ");
        for i in 0..128 {
            let a = 2.0 * (440.0 / 32.0);
            let b = 2.0_f32.powf( (i as f32 - 9.0) / 12.0 );
            freqs[i] = a * b / srate as f32;

            print!("{},", freqs[i]);
        }
        println!("");


        freqs
    }
}

impl jack::ProcessHandler for AudioHandler {
    fn process(&mut self, ctx: &jack::CallbackContext, nframes: jack::NumFrames) -> i32 {
        let output_buffer = self.output.get_write_buffer(nframes, &ctx);
        let input_buffer  = self.input.get_read_buffer(nframes, &ctx);

        let mut event_index = 0;
        let event_count = input_buffer.len();

        for i in 0..(nframes as usize) {
            if event_index < event_count {
                let event = input_buffer.get(event_index);

                println!("evi={} evt={}, i={}", event_index, event.get_jack_time(), i);
                if event.get_jack_time() == i as jack::NumFrames {
                    println!("got the frame");
                    let buf = event.raw_midi_bytes();

                    if buf[0] & 0x90 == 0x90 {
                        println!("note on!");
                        self.note    = buf[1];
                        self.note_on = 1.0;
                    } else if buf[0] & 0x90 == 0x80 {
                        println!("note off!");
                        self.note    = buf[1];
                        self.note_on = 0.0;
                    }
                    event_index += 1;
                    if event_index < event_count {
                        event_index += 1;
                    }
                }
            }

            self.ramp += self.note_freqs[self.note as usize];
            self.ramp = if self.ramp > 1.0 { self.ramp - 2.0 } else { self.ramp };

            let s = (2.0 * (consts::PI) * self.ramp).sin();
            output_buffer[i] = self.note_on*s;
            // println!("output_buffer[{}] = {}", i, output_buffer[i]);
        }

        match self.incoming.try_recv() {
            Ok(freqs) => self.note_freqs = freqs,
            Err(_) => (),
        };

        0
    }
}

extern "C" fn handle_sigint(_: i32) {
    RUNNING.store(false, atomic::Ordering::SeqCst);
}

fn main() {
    // register a signal handler (see comments at top of file)
    let action = signal::SigAction::new(
        signal::SigHandler::Handler(handle_sigint),
        signal::SaFlags::empty(),
        signal::SigSet::empty());

    unsafe { signal::sigaction(signal::Signal::SIGINT, &action) }.unwrap();

    // set our global atomic to true
    RUNNING.store(true, atomic::Ordering::SeqCst);


    let mut c = jack::Client::open("midi_sine", jack::options::NO_START_SERVER).unwrap();
    let i = c.register_input_midi_port("midi_in").unwrap();
    let o = c.register_output_audio_port("audio_out").unwrap();

    let (tx, rx) = mpsc::sync_channel(1);

    let handler = AudioHandler::new(i, o, rx);
    c.set_process_handler(handler).unwrap();

    let handler = MetadataHandler::new(tx);
    c.set_sample_rate_handler(handler).unwrap();

    c.activate().unwrap();

    while RUNNING.load(atomic::Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(1000));
    }
    println!("tearing down");
    c.close().unwrap();
}
