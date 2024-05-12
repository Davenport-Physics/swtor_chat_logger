
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;

use std::ffi::CStr;
use std::str;
use std::thread;
use std::time::Duration;

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

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref MESSAGES: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    static ref QUIT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

const CHAT_RELATIVE_ADDRESS: isize = 0x0ccd360;


#[ctor::ctor]
fn detour_init() {

    start_tcp_messager();
    start_quit_listener();
    begin_hook();

}

fn start_tcp_messager() {

    let mut stream = TcpStream::connect("127.0.0.1:4592").unwrap();
    stream.write(b"Hello, world!").unwrap();

    let messages = Arc::clone(&MESSAGES);
    thread::spawn(move || {

        loop {

            if QUIT.load(Ordering::Relaxed) {
                break;
            }

            for message in messages.lock().unwrap().iter() {
                stream.write(message.as_bytes()).unwrap();
            }
            thread::sleep(Duration::from_millis(1000));

        }

    });

}

fn submit_message(message: &str) {

    MESSAGES.lock().unwrap().push(message.to_string());

}

fn start_quit_listener() {

    thread::spawn(|| {

        let listener = TcpListener::bind("127.0.0.1:4593").unwrap();
        listener.accept().unwrap();

        QUIT.store(true, Ordering::Relaxed);
        unsafe {
            ChatHook.disable().unwrap();
        }

    });


}

fn begin_hook() {

    unsafe {

        match GetModuleHandleA(PCSTR(b"swtor.exe\0".as_ptr())) {
            Ok(hmodule) => {
                submit_message("Found module");
                submit_message(&format!("Module handle: {:?}", hmodule));
                begin_detour(hmodule.0 + CHAT_RELATIVE_ADDRESS);
            },
            Err(_) => {
                submit_message("Failed to find module");
            }
        }

    }

}

fn begin_detour(address: isize) {

    unsafe {

        let target: extern "C" fn(*mut u64, *const i8) = mem::transmute(address);
        match ChatHook.initialize(target, my_detour) {
            Ok(_) => {
                submit_message("Detour initialized");
                ChatHook.enable().unwrap();
            },
            Err(_) => {
                submit_message("Failed to initialize detour");
            }
        }

    }

}

fn my_detour(param_1: *mut u64, some_string: *const i8) {

    unsafe {

        let c_str: &CStr = CStr::from_ptr(some_string);
        let str_slice: &str = c_str.to_str().unwrap();

        if str_slice.to_lowercase().contains("</font>") {
            submit_message(str_slice);
        }

        ChatHook.call(param_1, some_string);

    }

}