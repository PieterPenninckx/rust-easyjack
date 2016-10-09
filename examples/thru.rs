extern crate easyjack as jack;
extern crate nix;

use nix::sys::signal;
use std::sync::atomic;
use std::thread;
use std::time::Duration;

// signals are unpleasant (check comments in simple_client example)
static RUNNING: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

type InputPort  = jack::InputPortHandle<jack::DefaultAudioSample>;
type OutputPort = jack::OutputPortHandle<jack::DefaultAudioSample>;

struct Connector {
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>
}

impl Connector {
    pub fn new(inputs: Vec<InputPort>, outputs: Vec<OutputPort>) -> Self {
        assert_eq!(inputs.len(), outputs.len());

        Connector {
            inputs: inputs,
            outputs: outputs
        }
    }
}

impl jack::ProcessHandler for Connector {
    fn process(&mut self, nframes: jack::NumFrames) -> i32 {
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

    let mut jack_client = jack::Client::open("testclient", jack::options::NO_START_SERVER).unwrap();
    println!("client created named: {}", jack_client.get_name());

    // 2 in, 2 out
    let input1 = jack_client.register_input_audio_port("input1").unwrap();
    let input2 = jack_client.register_input_audio_port("input2").unwrap();
    let output1 = jack_client.register_output_audio_port("output1").unwrap();
    let output2 = jack_client.register_output_audio_port("output2").unwrap();

    let handler = Connector::new(vec![input1, input2], vec![output1, output2]);
    jack_client.set_process_handler(handler).unwrap();

    // start everything up
    jack_client.activate().unwrap();

    // wait to get a SIGINT
    // jack will do all of its magic in other threads
    while RUNNING.load(atomic::Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(1000));
    }

    // now we can clean everything up
    // the library doesn't handle this for us because it would be rather confusing, especially
    // given how the underlying jack api actually works
    println!("tearing down");

    // closing the client unregisters all of the ports
    // unregistering the ports after the client is closed is an error
    jack_client.close().unwrap();
}
