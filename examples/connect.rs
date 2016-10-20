extern crate easyjack as jack;
extern crate getopts;

use getopts::Options;
use std::env;
use std::io::Write;
use std::io::stderr;
use std::process::exit;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

// bring the trait for all common port operations into scope
use jack::Port;

enum Mode {
    Connect(String, String),
    Disconnect(String, String),
}

/// An example of a struct wrapping a jack client
/// This pattern is overkill for this example program, but it serves to
/// demonstrate the pattern which the easyjack wrapper is designed to
/// accommodate.
/// The callback code communicates back to the main thread via a channel and
/// have the main thread performs actions
struct Connector<'a> {
    client: jack::Client<'a>,

    /// the incoming end of the channel running on the other thread
    /// The channel can receive messages composed of an Option of a pair of port ids
    incoming: Receiver<(jack::PortId, jack::PortId, jack::PortConnectStatus)>,
}

impl<'a> Connector<'a> {
    fn new(servername: Option<String>) -> Result<Self, jack::status::Status> {
        // we don't want to start a server if none is already started
        let opts   = jack::options::NO_START_SERVER;
        let myname = "connector";
        let client = match servername {
            None             => jack::Client::open(myname, opts),
            Some(servername) => jack::Client::open_connection_to(myname, &*servername, opts),
        };

        let client = match client {
            Ok(cl)    => cl,
            Err(code) => return Err(code)
        };

        // create our channel to communicate with the handler
        let (tx, rx) = mpsc::channel();

        // create the handler and give it the transmission end of the channel
        let handler = ConnectorHandler { outgoing: tx };

        // create an instance of the Connector, then set up the handler
        let mut conn = Connector { client: client, incoming: rx };
        conn.client.set_metadata_handler(handler).unwrap();

        Ok(conn)
    }

    fn activate(&mut self) -> Result<(), jack::status::Status> {
        self.client.activate()
    }

    fn connect(&mut self, port1: &str, port2: &str) -> Result<(), jack::status::Status> {
        self.client.connect_ports(port1, port2)
    }

    fn disconnect(&mut self, port1: &str, port2: &str) -> Result<(), jack::status::Status> {
        self.client.disconnect_ports(port1, port2)
    }

    fn wait_and_shutdown(self) {
        let (a, b, stat) = self.incoming.recv().unwrap();
        let n1 = self.client.get_port_by_id(a).unwrap().get_name();
        let n2 = self.client.get_port_by_id(b).unwrap().get_name();

        match stat {
            jack::PortConnectStatus::PortsConnected =>
                println!("ports connected: {} and {}", n1, n2),

            jack::PortConnectStatus::PortsDisconnected =>
                println!("ports disconnected: {} and {}", n1, n2)
        }
    }
}


/// the struct which will actually handle the jack callback
struct ConnectorHandler {
    outgoing: Sender<(jack::PortId, jack::PortId, jack::PortConnectStatus)>
}

impl jack::MetadataHandler for ConnectorHandler {
    fn on_port_connect(&mut self, a: jack::PortId, b: jack::PortId, status: jack::PortConnectStatus) {
        // a connection happened, this is probably the connection we made
        // if not, that's a shame, we don't handle this case
        // send a message to the channel so that the main thread knows it is
        // safe to shutdown
        self.outgoing.send( (a, b, status) ).unwrap();
    }

    fn callbacks_of_interest(&self) -> Vec<jack::MetadataHandlers> {
        vec![jack::MetadataHandlers::PortConnect]
    }
}

fn do_connect(server: Option<String>, mode: Mode) {
    // create a connector
    let mut connector = match Connector::new(server) {
        Ok(conn) => conn,
        Err(code) => {
            println!("could not create connector: {:?}", code);
            return
        }
    };

    // activate the connector
    match connector.activate() {
        Ok(()) => (),
        Err(code) => {
            println!("could not activate client: {:?}", code);
            return
        }
    }


    // make the connection (or disconnect some ports)
    match mode {
        Mode::Connect(p1, p2) => {
            match connector.connect(p1.as_str(), p2.as_str()) {
                Ok(())    => (),
                Err(code) => {
                    println!("Connect failed because: {:?}", code);
                    return
                }
            }
        },

        Mode::Disconnect(p1, p2) => {
            match connector.disconnect(p1.as_str(), p2.as_str()) {
                Ok(())    => (),
                Err(code) => {
                    println!("Disconnect failed because: {:?}", code);
                    return
                }
            }
        }
    }

    // wait for a bit, and shutdown
    connector.wait_and_shutdown();
}

fn usage_with_error(exe: String, err: &str, opts: Options) -> ! {
    let brief = format!("Error! {}\n{} port1 port2", err, opts.short_usage(&exe));
    let use_me = opts.usage(&brief);
    writeln!(&mut stderr(), "{}", use_me).unwrap();
    exit(1);
}

fn main() {
    // the main method contains a bunch of argument handling, don't worry very much about it!
    // setup the arg parser
    let mut opts = Options::new();
    opts.optopt("s", "servername", "name of the server to connect to", "NAME");

    // one of these is required, will check manually
    opts.optflag("c", "connect", "run in connect mode");
    opts.optflag("d", "disconnect", "run in disconnect mode");

    let args: Vec<String> = env::args().collect();
    let exe = args[0].clone();
    let matches = match opts.parse(&args[1..]) {
        Ok(m)  => m,
        Err(f) => panic!(f.to_string())
    };

    // do additional validation
    if !matches.opt_present("c") && !matches.opt_present("d") {
        usage_with_error(exe, "Did not specify either -c or -d", opts);
    }

    if matches.opt_present("c") && matches.opt_present("d") {
        usage_with_error(exe, "Cannot specify both -c and -d", opts);
    }

    if matches.free.len() != 2 {
        usage_with_error(exe, "Did not specify exactly 2 ports!", opts);
    }

    // set up some variables
    let port1 = matches.free[0].clone();
    let port2 = matches.free[1].clone();
    let server_name = matches.opt_str("s");

    let mode = if matches.opt_present("c") {
        Mode::Connect(port1, port2)
    } else {
        Mode::Disconnect(port1, port2)
    };

    do_connect(server_name, mode)
}
