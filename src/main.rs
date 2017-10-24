use std::thread;
use std::time::Duration;
use std::sync::mpsc::{self, TryRecvError};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::io;

extern crate ws;

use ws::{listen, Message};
use std::io::Write;
use std::io::Read;

extern crate structopt;
#[macro_use]
extern crate structopt_derive;

extern crate local_ip;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "buzzer", about = "Application server for buzzer.")]
struct Opt {

    #[structopt(help = "WebSocket Port", default_value="10000")]
    port: String,

    #[structopt(help = "If set to true, then there will be 3 second countdown before starting round",
        default_value="true")]
    countdown: bool
}

fn main() {
    let opt = Opt::from_args();
    let port = opt.port.clone();
    // TODO: Improve local_ip -- it returns wrong interface - add customizable regex
    println!("Buzzer server! Let's do some sick buzzin! \
        Possible local network ip: {}", local_ip::get().unwrap());
    let running = Arc::new(AtomicBool::new(false));
    let (send_to_loop, get_from_ws) = mpsc::channel();
    let run = running.clone();

    println!("Setting up websocket...");
    let ws = thread::spawn(move || {
        // TODO: Add info on player connecting/disconnecting (Move logic to separate handler)
        listen(format!("0.0.0.0:{}", port), |out| {
            let sender = send_to_loop.clone();
            let running = running.clone();
            move |msg: Message| {
                let cloned_msg = msg.clone();
                let split_msg: Vec<&str> = cloned_msg.as_text().unwrap().splitn(2, ':').collect();
                if split_msg.len() > 1 {
                    match split_msg[0] {
                        "connect" => { println!("WS: User connected: {}", split_msg[1]); out.send("connected") },
                        "win" => {
                            if running.load(Ordering::SeqCst) {
                                sender.send(msg);
                                // TODO: Jsonify response
                                out.send("won")
                            } else {
                                out.send("lost")
                            }
                        },
                        _ => { println!("WS: Unhandled message received: {}", split_msg[0]); out.send("unhandled") }
                    }
                } else {
                    println!("WS: Bad message received: {}", split_msg[0]); out.send("bad_msg")
                }
            }
        }).expect("Something went wrong when setting up ws.")
    });
    println!("Set up!");

    loop {
        println!("Press enter to start. Press q for shutdown.");
        let mut input = String::new();
        // TODO: Buffers inputs, what do?
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if input.contains("q") {
                    break;
                }
            }
            Err(error) => {
                println!("error: {}", error);
                break;
            }
        }

        if opt.countdown {
            println!("Starting in 3...");
            thread::sleep(Duration::from_millis(1000));
            println!("Starting in 2...");
            thread::sleep(Duration::from_millis(1000));
            println!("Starting in 1...");
            thread::sleep(Duration::from_millis(1000));
        }

        run.store(true, Ordering::SeqCst);
        // TODO: Send message to clients when it's possible to vote
        println!("GO!");

        println!("Waiting for winner...");
        let msg = get_from_ws.recv().expect("Something went wrong while receiving message");
        run.store(false, Ordering::SeqCst);
        println!("Winner is... {}!", msg);
    }
}
