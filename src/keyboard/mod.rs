#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_must_use)]

use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::os::unix::io::AsRawFd;
use std::mem;
use std::thread;
use std::sync::mpsc;
use std::option;
use regex::Regex;
use libc;
use std::os::unix::io::RawFd;
use std::ops::Drop;

const DEVICESDIR: &str = "/dev/input/";

fn get_keyboard_device_fnames() -> Vec<String> {
    let mut file = File::open("/proc/bus/input/devices").unwrap();
    let mut buffer: String = String::new();
    let reg = Regex::new(r"sysrq.*?kbd (event\d+)").unwrap();
    let mut fnames: Vec<String> = Vec::new();
    
    file.read_to_string(&mut buffer).unwrap();
    for capture in reg.captures_iter(&buffer) {
        fnames.push(DEVICESDIR.to_string() + &capture[1]);
    }

    fnames
}

#[test]
fn test_get_keyboard_device_fnames() {
    let fnames = get_keyboard_device_fnames();
    for fname in &fnames {
        println!("{}", fname);
    }
}

#[repr(C)]
struct InputEvnet {
    time: libc::timeval,
    ty: u16,
    code: u16,
    value: i32,
}

impl InputEvnet {
    fn new() -> InputEvnet {
        InputEvnet {
            time: libc::timeval { tv_sec: 0, tv_usec: 0 },
            ty: 0,
            code: 0,
            value: 0,
        }
    }
}

#[test]
#[ignore]
fn test_read_keyboard() {
    let fnames = get_keyboard_device_fnames();
    let mut file = File::open(fnames.last().unwrap()).unwrap();
    let mut event = InputEvnet {
        time: libc::timeval { tv_sec: 0, tv_usec: 0 },
        ty: 0,
        code: 0,
        value: 0,
    };
    unsafe {
        let event_p = &mut event as *mut InputEvnet as *mut libc::c_void;
        for i in 0..10 {
            libc::read(file.as_raw_fd(), event_p, mem::size_of::<InputEvnet>());
            println!("{}", &event.ty);
        }
    }
}

pub struct Keyboard {
    receiver: mpsc::Receiver<(u16, u16, i32)>,
    keyboard_handles: Vec<thread::JoinHandle<()>>,
    raw_fd: Vec<RawFd>,
    options: u32,
}

fn read_keyboard(device: File, sender: mpsc::Sender<(u16, u16, i32)>) {
    let mut event = InputEvnet::new();
    let event_p = &mut event as *mut InputEvnet as *mut libc::c_void;
    loop{
        unsafe {
            if libc::read(device.as_raw_fd(), event_p, mem::size_of::<InputEvnet>()) == -1 {
                return;
            }
        }
        sender.send((event.ty, event.code, event.value));
    }
}

extern "C" {
    fn ioctl_eviocgrab(fd: libc::c_int, mode: libc::c_int) -> libc::c_int;
}

pub mod OpenOption {
    pub const EVIOCGRAB: u32 = 0x1;
}
mod OpenOptionOfC {
    pub const EVIOCGRAB: u32 = 0x90;
}

impl Keyboard {
    pub fn open() -> Result<Keyboard, io::Error> {
        let mut keyboard_handles = Vec::new();
        let mut raw_fd: Vec<RawFd> = Vec::new();
        let (sender, receiver) = mpsc::channel();

        for fname in get_keyboard_device_fnames() {
            let device = File::open(fname)?;
            let child_sender = sender.clone();

            raw_fd.push(device.as_raw_fd());
            keyboard_handles.push(
                thread::spawn(move || read_keyboard(device, child_sender))
            );
        }

        Ok(Keyboard {
            keyboard_handles: keyboard_handles,
            receiver: receiver,
            raw_fd: raw_fd,
            options: 0,
        })
    }

    pub fn open_and_grab() -> Result<Keyboard, io::Error> {
        let mut kbd = Keyboard::open()?;
        let options = OpenOption::EVIOCGRAB;

        for fd in &kbd.raw_fd {
            unsafe {
                ioctl_eviocgrab(*fd, 1);
            }
        }
        
        kbd.options = options;

        Ok(kbd)
    }

    pub fn read(&self) -> (u16, u16, i32) {
        self.receiver.recv().unwrap()
    }

    fn read_when_state_is(&self, ty: u16) -> (u16, u16, i32) {
        loop {
            let (read_ty, code, state) = self.read();
            if(ty == read_ty){ return (ty, code, state) }
        }
    }

    pub fn read_key(&self) -> (u16, u16, i32) {
        return self.read_when_state_is(1);
    }

    pub fn read_syn(&self) -> (u16, u16, i32) {
        return self.read_when_state_is(0);
    }

    pub fn read_msc(&self) -> (u16, u16, i32) {
        return self.read_when_state_is(4);
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        if self.options & OpenOption::EVIOCGRAB == 1 {
            for fd in &self.raw_fd {
                unsafe {
                    ioctl_eviocgrab(*fd, 0);
                }
            }
        }
    }
}

#[cfg(test)]
use std::time;

#[test]
#[ignore]
fn test_keyboard() {
    let kbd = match Keyboard::open() {
        Ok(keyboard) => keyboard,
        Err(err) => {
            println!("You must use 'sudo'");
            return;
        }
    };
    thread::spawn(move || {
        loop {
            let (ty, code, value) = kbd.read();
            if value == 2 {
                continue;
            }
            println!( "\t{} {}", code, 
                      if value == 1 { "push" } else { "leave" } );
        }
    });
    let wait_duration = time::Duration::from_millis(3000);
    thread::sleep(wait_duration);
}

#[test]
#[ignore]
fn test_keyboard_loop() {
    let kbd = match Keyboard::open() {
        Ok(keyboard) => keyboard,
        Err(err) => {
            println!("You must use 'sudo'");
            return;
        }
    };
    println!("push q to exit");
    loop {
        let (ty, code, value) = kbd.read();
        if code == 16 {
            println!("end");
            break;
        }
        println!("\t{} {} {}", ty, code, value);
    }
}
