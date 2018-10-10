#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_must_use)]

use std::os::raw::*;
use std::ffi::{CStr, CString};
use std::collections::HashSet;

extern "C" {
    fn close_virtual_keyboard(fd: c_int) -> c_int;
    fn open_virtual_keyboard(name: *const c_char) -> c_int;
    fn key_input(fd: c_int, code: c_int) -> c_int;
    fn emit_event_sync(fd: c_int, ty: c_int, code: c_int, value: c_int) -> c_int;
    fn emit_key_event(fd: c_int, code: c_int, value: c_int) -> c_int;
    fn emit_event(fd: c_int, ty: c_int, code: c_int, value: c_int) -> c_int;
}


pub struct VirtualKeyboard {
    fd: c_int,
    pressed_keys: HashSet<u16>
}

impl VirtualKeyboard {
    pub fn new() -> Option<VirtualKeyboard> {
        let c_name = CString::new("virtual_keyboard").unwrap();
        let fd;
        unsafe { fd = open_virtual_keyboard(c_name.as_ptr()); }
        if fd == -1 {
            return None;
        }

        Some(VirtualKeyboard {
            fd: fd,
            pressed_keys: HashSet::new(),
        })
    }

    pub fn emit_event(&self, ty: u16, code: u16, state: i32) -> Option<()> {
        if(unsafe { emit_event(self.fd, ty as i32, code as i32, state) } == -1) {
            return None;
        }
        return Some(());
    }

    pub fn emit_event_sync(&self, ty: u16, code: u16, state: i32) -> Option<()> {
        if(unsafe { emit_event_sync(self.fd, ty as i32, code as i32, state) } == -1) {
            return None;
        }
        return Some(());
    }

    pub fn push(&mut self, code: u16) -> Option<()> {
        if(unsafe { emit_key_event(self.fd, code as i32, 1) } == -1) {
            return None;
        }
        self.pressed_keys.insert(code);
        return Some(());
    }

    pub fn leave(&mut self, code: u16) -> Option<()> {
        if(unsafe { emit_key_event(self.fd, code as i32, 0) } == -1) {
            return None;
        }
        self.pressed_keys.remove(&code);
        return Some(());
    }

    pub fn repeat(&self, code: u16) -> Option<()> {
        if(unsafe { emit_key_event(self.fd, code as i32, 2) } == -1) {
            return None;
        }
        return Some(());
    }

    pub fn contains(&self, code: u16) -> bool {
        self.pressed_keys.contains(&code)
    }
}


/* test */
#[cfg(test)]
mod test {
    use std::time::Duration;
    use std::thread;
    use super::VirtualKeyboard;

    extern "C" {
        fn perror();
    }

    #[test]
    #[ignore]
    fn test_kbd_input() {
        let sleep_time = Duration::from_millis(500);
        let mut kbd = match VirtualKeyboard::new() { 
            Some(kbd) => kbd, 
            None => { unsafe { perror(); } return; } 
        };
/*+ 以下のキーコードは使えない +*/
//        thread::sleep(sleep_time);
//        kbd.push(keycodes::KEY_LEFTSHIFT);
//        for i in 0..5 {
//            kbd.push(keycodes::KEY_A);
//            kbd.leave(keycodes::KEY_A);
//            thread::sleep(sleep_time);
//        }
//        kbd.leave(keycodes::KEY_LEFTSHIFT);
    }
}
