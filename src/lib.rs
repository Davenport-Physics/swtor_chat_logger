
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Mutex;

use std::ffi::CStr;
use std::str;
use std::thread;

use log::info;
use retour::static_detour;
use retour::StaticDetour;
use std::mem;

use windows::Win32::System::LibraryLoader::{GetModuleHandleExA, GetModuleHandleA};
use windows::core::PCSTR;
use windows::Win32::Foundation::HMODULE;

static_detour! {

    static ChatHook: extern "C" fn(*mut u64, *const i8);

}

const CHAT_RELATIVE_ADDRESS: isize = 0x0ccd360;


#[ctor::ctor]
fn detour_init() {

    let mut stream = TcpStream::connect("127.0.0.1:4592").unwrap();
    stream.write(b"Hello, world!").unwrap();

    tracing_subscriber::fmt()
        .with_writer(Mutex::new(stream))
        .init();

    info!("Things are well");
    start_quit_listener();

    begin_hook();

}

fn start_quit_listener() {

    thread::spawn(|| {

        let listener = TcpListener::bind("127.0.0.1:4593").unwrap();
        listener.accept().unwrap();

        unsafe {
            ChatHook.disable().unwrap();
        }

    });


}

fn begin_hook() {

    unsafe {

        match GetModuleHandleA(PCSTR(b"swtor.exe\0".as_ptr())) {
            Ok(hmodule) => {
                info!("Found module");
                info!("Module handle: {:?}", hmodule);
                begin_detour(hmodule.0 + CHAT_RELATIVE_ADDRESS);
            },
            Err(_) => {
                info!("Failed to find module");
            }
        }

    }

}

fn begin_detour(address: isize) {

    unsafe {

        let target: extern "C" fn(*mut u64, *const i8) = mem::transmute(address);
        match ChatHook.initialize(target, my_detour) {
            Ok(_) => {
                info!("Detour initialized");
                ChatHook.enable().unwrap();
            },
            Err(_) => {
                info!("Failed to initialize detour");
            }
        }

    }

}

fn my_detour(param_1: *mut u64, some_string: *const i8) {

    unsafe {

        let c_str: &CStr = CStr::from_ptr(some_string);
        let str_slice: &str = c_str.to_str().unwrap();

        if str_slice.to_lowercase().contains("</font>") {
            info!("Chat message: {}", str_slice);
        }

        ChatHook.call(param_1, some_string);

    }

}