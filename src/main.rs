extern crate libc;
extern crate regex;
#[macro_use]
extern crate lazy_static;
use std::env;
use std::fs::File;

mod keyboard;
mod virtual_keyboard;
mod rules;
mod key_converter;

use keyboard::Keyboard;
use virtual_keyboard::*;
use key_converter::KeyConverter;



fn main() {
    let wait_time = std::time::Duration::from_millis(250);
    let mut show_state = false;
    let mut filename = None;

    // 引数をパースする
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "-s" | "--show-stats" => show_state = true,
            s => filename = Some(s.to_string()),
        }
    }

    let mut kc = match filename {
        Some(f) => match File::open(f) {
            Ok(f) => match KeyConverter::new(f) {
                Some(kc) => kc,
                None => return println!("Error: ルールが間違っています。")
            },
            Err(_) => return println!("Error: ファイルが開けません")
        },
        None => return println!("Error: ファイル名がありません")
    };

    std::thread::sleep(wait_time);
    
    let kbd = match Keyboard::open_and_grab() {
        Ok(kbd) => kbd,
        Err(e) => { println!("Error: {:?}", e.kind()); return; }
    };

    let mut vkbd = match VirtualKeyboard::new() {
        Some(vkbd) => vkbd,
        None => {
            println!("Can't create virtual_keyboard.");
            return;
        }
    };

    // 最後にpushしたキーコードを入れておく
    let mut last_push = 0;

    //loop {
    loop {
        let (_, read_code, state) = kbd.read_key();
        
        // 結果をoptionで受け取る
        let (push, leave) = match state {
            // push
            1 => {
                let (push, leave) = kc.push(read_code);
                last_push = *push.last().unwrap();

                for l in &leave {
                    vkbd.leave(*l);
                }

                for p in &push {
                    vkbd.push(*p);
                }

                (Some(push), Some(leave))
            },
            // leave
            0 => {
                let leave = kc.leave(read_code);

                for l in &leave {
                    vkbd.leave(*l);
                }

                (None, Some(leave))
            },
            // repeat
            2 => {
                // 最後にvkbdにpushされたキーコードをrepeatする
                vkbd.repeat(last_push);

                (None, None)
            },
            _ => panic!()
        };

        // キーの状態を表示する
        if show_state {
            // 現実世界のキーボードで入力された値を表示
            print!("\t kbd | ");
            match state {
                0 => print!("leave "),
                1 => print!("push "),
                2 => print!("repeat "),
                _ => ()
            }
            println!("{}", read_code);

            // 仮想的なキーボードで入力された値を表示
            print!("\tvkbd | ");
            for p in push.unwrap_or(Vec::new()) {
                print!("push {} ", p);
            }
            for l in leave.unwrap_or(Vec::new()) {
                print!("leave {} ", l);
            }
            if state == 2 {
                print!("repeat {} ", last_push);
            }
            println!();

            // 押されているキーをルールに適用した結果を表示
            println!("\trule | {}", kc.filter_to_string());
            println!()
        }
    }
}