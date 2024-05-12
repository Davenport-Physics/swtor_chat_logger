
use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::Mutex;

use log::info;

use windows::Win32::System::LibraryLoader::GetModuleHandleExA;
use windows::core::PCSTR;
use windows::Win32::Foundation::HMODULE;

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

    let mut phmodule: HMODULE = HMODULE::default();
    unsafe {

        match GetModuleHandleExA(0, PCSTR(b"hook_test.exe\0".as_ptr()), &mut phmodule as *mut _) {
            Ok(_) => {
                info!("Found module");
                info!("At maybe this ? {:?}", phmodule.0);
            },
            Err(_) => {
                info!("Failed to find module");
            }
        }

    }

}

fn my_add(a: i32, b: i32) -> i32 {
    a * 2 + b
}