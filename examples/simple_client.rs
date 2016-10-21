// use the jack bindings
extern crate easyjack as jack;

// use nix for signal handling
// we need to use low level signal handlers to clean up in real time (the other crates don't behave
// in real time very well)
// This is a pain, but rust doesn't currently have a better way to handle signals without causing
// stuttering (xruns) as the client is shutting down
extern crate nix;

use nix::sys::signal;
use std::sync::atomic;
use std::sync::mpsc::{SyncSender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// the size of a list of samples
const N: usize = 200;

// signals are literally a giant steaming mess
// create an atomic bool which will be save to change the value of inside of a signal handler
static RUNNING: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

/// This struct handles the process callback
/// It holds a list of samples and continues to play them back until it receives a new set of
/// samples over the `incoming` channel. When it receives new samples, it moves them (memcpy) into
/// its own buffer.
/// The samples are played back at different rates so that we can hear the difference in the right
/// and left channel
struct AudioHandler {
    /// current list of samples
    samples: [jack::DefaultAudioSample; N],

    /// where we currently are in our list of samples
    right_phase: usize,
    left_phase: usize,

    /// handles for the ports we are reading and writing to.
    /// Remember, port handles can become invalid if you are careless with them
    right_output: jack::OutputPortHandle<jack::DefaultAudioSample>,
    left_output: jack::OutputPortHandle<jack::DefaultAudioSample>,

    /// incoming changes
    /// these are copied into the channel, then copied out of the channel.
    /// Copies are pretty fast, but some applications may need to be more clever
    incoming: Receiver<[jack::DefaultAudioSample; N]>,
}

impl AudioHandler {
    /// constructs a new audio handler
    fn new(
        init_samples: [jack::DefaultAudioSample; N],
        right: jack::OutputPortHandle<jack::DefaultAudioSample>,
        left: jack::OutputPortHandle<jack::DefaultAudioSample>,
        incoming: Receiver<[jack::DefaultAudioSample; N]>) -> Self
    {
        AudioHandler {
            samples: init_samples,
            right_phase: 0,
            left_phase: 0,
            right_output: right,
            left_output: left,
            incoming: incoming
        }
    }
}

/// implement the `ProcessHandler` for the `AudioHandler`
impl jack::ProcessHandler for AudioHandler {
    fn process(&mut self, ctx: &jack::CallbackContext, nframes: jack::NumFrames) -> i32 {
        // get the ports
        let right = self.right_output.get_write_buffer(nframes, ctx);
        let left  = self.left_output.get_write_buffer(nframes, ctx);

        // for every frame, write our current progress
        for i in 0..(nframes as usize) {
            right[i] = self.samples[self.right_phase];
            left[i]  = self.samples[self.left_phase];

            // adjust the phases, keep left and right out of sync
            self.right_phase = self.right_phase + 1;
            self.left_phase  = self.left_phase + 3;

            if self.right_phase >= self.samples.len() {
                self.right_phase = 0
            }

            if self.left_phase >= self.samples.len() {
                self.left_phase = 0
            }
        }

        // try to update the samples, if we need to
        match self.incoming.try_recv() {
            Ok(samples) => self.samples = samples,
            Err(_) => (),
        };

        0
    }
}

/// A simple wrapper around a jack client
/// Creates a handler and sets up channels to communicate with the handler
struct SimpleClient<'a> {
    client: jack::Client<'a>,
    sender: SyncSender<[jack::DefaultAudioSample; N]>,
}

impl<'a> SimpleClient<'a> {
    fn new() -> Result<Self, jack::status::Status> {
        let client = jack::Client::open("simple", jack::options::NO_START_SERVER);
        let mut client = match client {
            Ok((client, _)) => client,
            Err(code)       => return Err(code),
        };

        // get some ports
        let right = client.register_output_audio_port("output1").unwrap();
        let left  = client.register_output_audio_port("output2").unwrap();

        // create a channel pair we can use to communicate with
        let (tx, rx) = mpsc::sync_channel(1);

        // create a client, set it up as an audio processing handler
        let handler = AudioHandler::new(SimpleClient::compute_sine(0.2), right, left, rx);
        client.set_process_handler(handler).unwrap();

        Ok(SimpleClient {
            client: client,
            sender: tx,
        })
    }

    fn activate(&mut self) -> Result<(), jack::status::Status> {
        self.client.activate()
    }

    fn run(mut self) {
        let mut i = 0;
        while RUNNING.load(atomic::Ordering::SeqCst) {
            let newsine = SimpleClient::compute_sine(i as f32 / 10.0);

            match self.sender.send(newsine) {
                Ok(_)  => (),
                Err(_) => (),
            };

            i += 1;
            if i > 10 {
                i = 0
            }

            thread::sleep(Duration::from_millis(1000));
        }

        println!("tearing down");
        self.client.close().unwrap();
    }

    fn compute_sine(constant: f32) -> [jack::DefaultAudioSample; N] {
        let mut sine = [0.0; N];
        for i in 0..N {
            let inner = ((i as f32) / (N as f32)) * 3.14159265 * 2.0;
            sine[i] = constant * inner.sin();
        }

        sine
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

    let mut c = SimpleClient::new().unwrap();
    c.activate().unwrap();
    c.run()
}
