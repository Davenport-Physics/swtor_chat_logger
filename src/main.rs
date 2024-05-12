use dll_syringe::{Syringe, process::OwnedProcess};

use std::io::{prelude::*, stdin};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

mod pid_finder;

#[macro_use]
extern crate lazy_static;

fn main() {

    let target_pid = pid_finder::get_pid();

    match target_pid {
        Ok(_) => {},
        Err(_) => {
            println!("Failed to find SWTOR process");
            return;
        }
    }

    let target_process = OwnedProcess::from_pid(target_pid.unwrap()).unwrap();
    let syringe = Syringe::for_process(target_process);

    thread::spawn(|| {

        let listener = TcpListener::bind("127.0.0.1:4592").unwrap();

        let mut buffer: [u8; 1024] = [0; 1024];
        let mut stream = listener.accept().unwrap().0;
        println!("Listening for bytes");
        loop {

            stream.read(&mut buffer).unwrap();
            println!("{}", String::from_utf8_lossy(&buffer));

        }

    });
    
    let injected_payload = syringe.inject("./target/debug/swtor_chat_capturer.dll").unwrap();
    stdin().read_line(&mut String::new()).unwrap();

    let mut stream = TcpStream::connect("127.0.0.1:4593").unwrap();
    stream.write(b"Hello, world!").unwrap();

    println!("Waiting for 5 seconds");
    thread::sleep(Duration::from_secs(5));

    println!("Disabling hook");
    syringe.eject(injected_payload).unwrap();

}
