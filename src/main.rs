extern crate libc;
extern crate regex;
#[macro_use]
extern crate lazy_static;
use std::collections::{HashMap, HashSet};
use std::env;

mod keyboard;
mod virtual_keyboard;
mod rules;

use keyboard::Keyboard;
use virtual_keyboard::*;
use rules::Rules;



fn main() {
    let wait_time = std::time::Duration::from_millis(250);
    let path = match env::args().nth(1) {
        Some(arg) => { arg },
        None => { 
            println!("There is no option.");
            return;
        }
    };

    std::thread::sleep(wait_time);
    let kbd = match Keyboard::open_and_grab() {
        Ok(kbd) => kbd,
        Err(e) => { println!("{:?}", e.kind()); return; }
    };
    let mut vkbd = match VirtualKeyboard::new() {
        Some(vkbd) => vkbd,
        None => {
            println!("Can't create virtual_keyboard.");
            return;
        }
    };
    let rules = match Rules::from_file(&path) { 
        Some(rules) => rules,
        None => return
    };


    let mut pressed_keys: HashSet<u16> = HashSet::new();

    loop {
        let (ty, read_code, state) = kbd.read_key();

        #[cfg(debug_assertions)]
        println!("\t{} {} {}", ty, read_code, state);

        let code = rules.change_keycode(read_code).unwrap_or(read_code);

        match state {
            1 => {
                pressed_keys.insert(code);
                if let Some(rule) = rules.contains_and_trigger(&pressed_keys, code) {
                    for key in &rule.keys {
                        // 必要のないキーが押されていたら離す
                        if vkbd.contains(*key) {
                            vkbd.leave(*key); 
                        }
                    }

                    for value in &rule.value {
                        vkbd.push(*value);
                    }
                }else{
                    vkbd.push(code);
                }
            },
            0 => {
                if let Some(rule) = rules.contains_and_trigger(&pressed_keys, code) {
                    for value in &rule.value {
                        if vkbd.contains(*value) == true {
                            // 念の為、押されているキーのみを離す
                            vkbd.leave(*value);
                        }
                    }

                }else if vkbd.contains(code) { // 入力されていないキーは戻さない
                    vkbd.leave(code);
                }
                pressed_keys.remove(&code);
            },
            2 => {
                if let Some(rule) = rules.contains_and_trigger(&pressed_keys, code) {
                     vkbd.repeat(rule.value[0]); // 最初のキーのみリピートする
                }else if vkbd.contains(code) {
                    vkbd.repeat(code);
                }
            },
            _ => {}
        }
    }
}
