
use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::Mutex;

use log::info;
use retour::static_detour;
use retour::StaticDetour;
use std::mem;

use windows::Win32::System::LibraryLoader::{GetModuleHandleExA, GetModuleHandleA};
use windows::core::PCSTR;
use windows::Win32::Foundation::HMODULE;

static_detour! {

    static Test: fn(i32, i32) -> i32;

}


#[ctor::ctor]
fn detour_init() {

    let mut stream = TcpStream::connect("127.0.0.1:4592").unwrap();
    stream.write(b"Hello, world!").unwrap();

    tracing_subscriber::fmt()
        .with_writer(Mutex::new(stream))
        .init();

    begin_hook();

}

fn begin_hook() {

    const RELATIVE_ADDRESS: isize = 0x0002af0;

    unsafe {

        match GetModuleHandleA(PCSTR(b"hook_test.exe\0".as_ptr())) {
            Ok(hmodule) => {
                info!("Found module");
                info!("Module handle: {:?}", hmodule);
                begin_detour(hmodule.0 + RELATIVE_ADDRESS);
            },
            Err(_) => {
                info!("Failed to find module");
            }
        }

    }

}

fn begin_detour(address: isize) {

    unsafe {
        let target: fn(i32, i32) -> i32 = mem::transmute(address);
        match Test.initialize(target, my_add) {
            Ok(_) => {
                info!("Detour initialized");
                Test.enable().unwrap();
            },
            Err(_) => {
                info!("Failed to initialize detour");
            }
        }
    }

}

fn my_add(a: i32, b: i32) -> i32 {
    a * 2 + b
}