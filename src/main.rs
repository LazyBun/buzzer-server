use std::thread;
use std::time::Duration;
use std::sync::mpsc::{self, TryRecvError};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::io;

extern crate ws;

use ws::listen;

fn main() {
    println!("Buzzer server! Let's do some sick buzzin!");
    let running = Arc::new(AtomicBool::new(false));
    let (send_to_loop, get_from_ws) = mpsc::channel();
    let run = running.clone();

    println!("Setting up websocket...");
    let ws = thread::spawn(move || {
        listen("127.0.0.1:3012", |out| {
            let sender = send_to_loop.clone();
            let running = running.clone();
            move |msg| {
                if running.load(Ordering::SeqCst) {
                    sender.send(msg);
                    out.send("true")
                } else {
                    out.send("false")
                }

            }
        }).expect("Something went wrong when setting up ws.")
    });
    println!("Set up!");

    loop {
        println!("Press enter to start. Press q for shutdown.");
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if input == "q" {
                    break;
                }
            }
            Err(error) => {
                println!("error: {}", error);
                break;
            }
        }
        println!("Starting in 3...");
        thread::sleep(Duration::from_millis(1000));
        println!("Starting in 2...");
        thread::sleep(Duration::from_millis(1000));
        println!("Starting in 1...");
        thread::sleep(Duration::from_millis(1000));
        run.store(true, Ordering::SeqCst);
        println!("GO!");

        println!("Waiting for winner...");
        let msg = get_from_ws.recv().expect("Something went wrong while receiving message");
        run.store(false, Ordering::SeqCst);
        println!("Winner is... {}!", msg);
    }
}
