
use std::sync::{Arc, Mutex};
use windows::Win32::Foundation::{HWND, LPARAM, BOOL, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, 
    GetWindowThreadProcessId,
    GetWindowTextW
};

lazy_static! {
    static ref SWTOR_PID: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
}

unsafe extern "system" fn enum_windows_existing_proc(hwnd: HWND, _param1: LPARAM) -> BOOL {

    let mut text: [u16; 256] = [0; 256];
    GetWindowTextW(hwnd, &mut text);

    let window_text: String;
    match String::from_utf16(&text) {
        Ok(text) => {
            window_text = text.replace("\0", "");
        },
        Err(_) => {
            return BOOL(1);
        }
    }

    if window_text == "Star Wars™: The Old Republic™" {

        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id as *mut u32));
        SWTOR_PID.lock().unwrap().replace(process_id);
        return BOOL(0);

    }

    return BOOL(1);

}

pub fn get_pid() -> Result<u32, &'static str> {

    unsafe {

        match EnumWindows(Some(enum_windows_existing_proc), LPARAM(0)) {

            Ok(_) => {
                return Err("Failed to enumerate windows");
            },
            Err(_) => {
                return Ok(SWTOR_PID.lock().unwrap().unwrap());
            }

        }
        
    }

}