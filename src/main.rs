extern crate libc;
extern crate regex;
#[macro_use]
extern crate lazy_static;
use std::env;
use std::fs::File;

mod keyboard;
mod virtual_keyboard;
mod key_converter;

use keyboard::Keyboard;
use virtual_keyboard::*;
use key_converter::KeyConverter;


fn loop_keymap(kbd: Keyboard, 
               mut vkbd: VirtualKeyboard, 
               mut kc: KeyConverter, 
               show_state: bool) 
    {
    let mut last_push = None;

    loop {
        let (_, read_code, state) = kbd.read_key();
        
        // 結果をoptionで受け取る
        let (push, leave) = match state {
            // push
            1 => {
                let (push, leave) = kc.push(read_code);
                if let Some(p) = push.last() {
                    last_push = Some(*p);
                } else {
                    last_push = None;
                }

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
                if let Some(p) = last_push {
                    vkbd.repeat(p);
                }

                (None, None)
            },
            _ => panic!()
        };

        // キーの状態を表示する
        if show_state {
            // 現実世界のキーボードで入力された値を表示
            print!("\t{:>15} | ", "kbd");
            match state {
                0 => print!("leave "),
                1 => print!("push "),
                2 => print!("repeat "),
                _ => ()
            }
            println!("{}", read_code);

            // 仮想的なキーボードで入力された値を表示
            print!("\t{:>15} | ", "vkbd");
            for p in push.unwrap_or(Vec::new()) {
                print!("push {} ", p);
            }
            for l in leave.unwrap_or(Vec::new()) {
                print!("leave {} ", l);
            }
            if state == 2 {
                if let Some(p) = last_push {
                    print!("repeat {} ", p);
                }
            }
            println!();

            // 押されているキーをルールに適用した結果を表示
            let name = format!("@{}", kc.get_rules_name());
            println!("\t{:>15} | {}", name, kc.filter_to_string());

            println!()
        }
    }
}

// 実際にvkbdでは入力しない
fn loop_keymap_without_vkbd(kbd: Keyboard, mut kc: KeyConverter) {
    loop {
        let (_, read_code, state) = kbd.read_key();
        
        // 結果をoptionで受け取る
        match state {
            // push
            1 => {
                kc.push(read_code);
            },
            // leave
            0 => {
                kc.leave(read_code);
            },
            // repeat
            2 => (),
            _ => panic!()
        }

        // キーの状態を表示する
        // 現実世界のキーボードで入力された値を表示
        print!("\t{:>15} | ", "kbd");
        match state {
            0 => print!("leave "),
            1 => print!("push "),
            2 => print!("repeat "),
            _ => ()
        }
        println!("{}", read_code);

        // 仮想的なキーボードで入力された値を表示しない
        println!("\t{:>15} | ", "vkbd");

        // 押されているキーをルールに適用した結果を表示
        let name = format!("@{}", kc.get_rules_name());
        println!("\t{:>15} | {}", name, kc.filter_to_string());

        println!()
    }
}

fn print_help() {
    println!("usage:");
    println!("    keymap [options...] <rule>");
    println!();
    println!("arguments:");
    println!("    <rule>    ルールを記述したファイルを指定します");
    println!();
    println!("options:");
    println!("    -s, --show-stats    実行中にキーの状態を出力します");
    println!("    -r, --rule          ルールを適用しますが、実際に変換後のキーが入力されることはありません");
}

fn print_error<T: std::fmt::Display>(t: T) {
    println!("Error: {}", t);
}

fn main() {
    let wait_time = std::time::Duration::from_millis(250);
    let mut show_state = false;
    let mut only_rule = false;
    let mut filename = None;

    // 引数をパースする
    for arg in env::args().skip(1) {
        if arg.len() == 0 {
            continue
        }
        
        if arg.get(..2) == Some("--") {
            match arg.get(2..) {
                Some("show-state") => show_state = true,
                Some("rule") => only_rule = true,
                _ => {
                    print_error(format!("'{}'は無効なオプションです", arg));
                    print_help();
                    return 
                }
            }
            continue
        }

        if arg.chars().next() == Some('-') {
            for c in arg.chars().skip(1) {
                match c {
                    's' => show_state = true,
                    'r' => only_rule = true,
                    _ => {
                        print_error(format!("'{}'は無効なオプションです", arg));
                        print_help();
                        return 
                    }
                }
            }
            continue
        }

        filename = Some(arg);
    }

    let kc = match filename {
        Some(f) => match File::open(f) {
            Ok(f) => match KeyConverter::new(f) {
                Ok(kc) => kc,
                Err(e) => return print_error(e)
            },
            Err(_) => return print_error("ファイルが開けません")
        },
        None => {
            print_error("ファイル名がありません");
            print_help();
            return 
        }
    };

    std::thread::sleep(wait_time);
    
    let kbd;
    if only_rule {
        // grabしない
        kbd = match Keyboard::open() {
            Ok(kbd) => kbd,
            Err(e) => { print_error(format!("{:?}", e.kind())); return; }
        };
    } else {
        kbd = match Keyboard::open_and_grab() {
            Ok(kbd) => kbd,
            Err(e) => { print_error(format!("{:?}", e.kind())); return; }
        };
    }

    let vkbd = match VirtualKeyboard::new() {
        Some(vkbd) => vkbd,
        None => {
            print_error("Can't create virtual_keyboard.");
            return;
        }
    };

    if only_rule {
        loop_keymap_without_vkbd(kbd, kc);
    } else {
        loop_keymap(kbd, vkbd, kc, show_state);
    }
}