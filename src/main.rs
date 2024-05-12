use dll_syringe::{Syringe, process::OwnedProcess};
use rusqlite::{params, Connection, Result};

use std::io::{prelude::*, stdin};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod pid_finder;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref MESSAGES: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    static ref QUIT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

fn main() {


    init_database();

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

        let messages = Arc::clone(&MESSAGES);

        let listener = TcpListener::bind("127.0.0.1:4592").unwrap();

        let mut buffer: [u8; 1024] = [0; 1024];
        let mut stream = listener.accept().unwrap().0;
        println!("Listening for bytes");
        loop {

            stream.read(&mut buffer).unwrap();
            messages.lock().unwrap().push(String::from_utf8_lossy(&buffer).to_string());
            buffer = [0; 1024];

        }

    });
    
    let injected_payload = syringe.inject("./target/debug/swtor_chat_capturer.dll").unwrap();
    stdin().read_line(&mut String::new()).unwrap();

    let mut stream = TcpStream::connect("127.0.0.1:4593").unwrap();
    stream.write(b"Hello, world!").unwrap();

    QUIT.store(true, Ordering::Relaxed);

    println!("Waiting for 5 seconds");
    thread::sleep(Duration::from_secs(5));

    println!("Disabling hook");
    syringe.eject(injected_payload).unwrap();

}

fn init_database() {

    let conn = Connection::open("swtor_chat.db").unwrap();

    const TABLE: &str = 
    "
        CREATE TABLE IF NOT EXISTS chat
        (
            id INTEGER PRIMARY KEY,
            message TEXT NOT NULL,
            timestamp TIMESTAMP NOT NULL DEFAULT(CURRENT_TIMESTAMP)
        );
    ";

    conn.execute(TABLE, params![]).unwrap();

    const INSERT_QUERY: &str = 
    "
        INSERT INTO chat (message) VALUES (?);
    ";

    thread::spawn(move || {

        thread::sleep(Duration::from_secs(1));
        let mut stmt = conn.prepare(INSERT_QUERY).unwrap();

        let messages = Arc::clone(&MESSAGES);
        loop {

            if QUIT.load(Ordering::Relaxed) {
                break;
            }

            for message in messages.lock().unwrap().drain(..) {
                stmt.execute(params![message]).unwrap();
            }

            thread::sleep(Duration::from_secs(1));

        }

    });

}
