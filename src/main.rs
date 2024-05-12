use dll_syringe::{Syringe, process::OwnedProcess};

use std::io::{prelude::*, stdin};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;


fn main() {

    let target_process = OwnedProcess::find_first_by_name("hook_test.exe").unwrap();
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
    
    let injected_payload = syringe.inject("./target/debug/hooking_dll.dll").unwrap();
    thread::sleep(Duration::from_secs(10));
    stdin().read_line(&mut String::new()).unwrap();
    syringe.eject(injected_payload).unwrap();

}
