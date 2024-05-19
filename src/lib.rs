
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

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::mem;

use windows::Win32::System::LibraryLoader::{GetModuleHandleExA, GetModuleHandleA};
use windows::core::PCSTR;
use windows::Win32::Foundation::HMODULE;

static_detour! {
    static ChatHook: extern "C" fn(*mut u64, *const *const i8, *const *const i8, i32, *const *const i8) -> i64;
}

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref MESSAGES: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    static ref QUIT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

const CHAT_RELATIVE_ADDRESS: isize = 0x03f3380;

#[derive(Serialize, Deserialize)]
pub enum MessageType {
    Info,
    Chat
}


#[ctor::ctor]
fn detour_init() {

    start_tcp_messager();
    start_quit_listener();
    begin_hook();

}

fn start_tcp_messager() {

    let mut stream = TcpStream::connect("127.0.0.1:4592").unwrap();

    let messages = Arc::clone(&MESSAGES);
    thread::spawn(move || {

        loop {

            if QUIT.load(Ordering::Relaxed) {
                break;
            }

            for message in messages.lock().unwrap().drain(..) {
                stream.write(message.as_bytes()).unwrap();
            }
            thread::sleep(Duration::from_millis(100));

        }

    });

}

fn submit_message(message_type: MessageType, message: &str) {

    MESSAGES.lock().unwrap().push(json!({
        "type": message_type,
        "message": message.to_string()
    }).to_string());

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
                submit_message(MessageType::Info, "Found module");
                submit_message(MessageType::Info, &format!("Module handle: {:?}", hmodule));
                begin_detour(hmodule.0 + CHAT_RELATIVE_ADDRESS);
            },
            Err(_) => {
                submit_message(MessageType::Info, "Failed to find module");
            }
        }

    }

}

fn begin_detour(address: isize) {

    unsafe {

        let target: extern "C" fn(*mut u64, *const *const i8, *const *const i8, i32, *const *const i8) -> i64 = mem::transmute(address);
        match ChatHook.initialize(target, my_detour) {

            Ok(_) => {

                submit_message(MessageType::Info, "Detour initializing");

                if let Err(err) = ChatHook.enable() {
                    submit_message(MessageType::Info, &format!("Failed to enable detour: {:?}", err));
                    return;
                }

                submit_message(MessageType::Info, "Detour enabled");
            },
            Err(_) => {
                submit_message(MessageType::Info, "Failed to initialize detour");
            }
        }

    }

}

fn my_detour(param_1: *mut u64, from_character_id: *const *const i8, to_character_id: *const *const i8, channel_id: i32, chat_message: *const *const i8) -> i64 {

    unsafe {

        let t_from_character_id: &CStr = CStr::from_ptr(*from_character_id);
        //submit_message(MessageType::Info, &format!("{:?}", t_from_character_id.to_string_lossy()));
        match t_from_character_id.to_str() {
            Ok(s) => {
                submit_message(MessageType::Chat, &format!("Converted from_character_id: {}", s));
            },
            Err(err) => {
                submit_message(MessageType::Info, &format!("Failed to convert from_character_id, {:?}", err));
            }
        }

        let t_to_character_id: &CStr = CStr::from_ptr(*to_character_id);

        match t_to_character_id.to_str() {
            Ok(s) => {
                submit_message(MessageType::Chat, &format!("Converted to_character_id: {}", s));
            },
            Err(err) => {
                submit_message(MessageType::Info, &format!("Failed to convert to_character_id {:?}", err));
            }
        }

        let t_chat_message: &CStr = CStr::from_ptr(*chat_message);

        match t_chat_message.to_str() {
            Ok(s) => {
                submit_message(MessageType::Chat, &format!("Converted chat_message: {}", s));
            },
            Err(err) => {
                submit_message(MessageType::Info, &format!("Failed to convert chat_message {:?}", err));
            }
        }

        submit_message(MessageType::Info, &format!("channel_id: {}", channel_id));

        return ChatHook.call(param_1, from_character_id, to_character_id, channel_id, chat_message);

    }

}