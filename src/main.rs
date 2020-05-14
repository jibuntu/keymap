extern crate libc;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod keyboard;
mod virtual_keyboard;
mod rules;
mod key_converter;

use keyboard::Keyboard;
use virtual_keyboard::*;
use key_converter::KeyConverter;



fn main() {
    let wait_time = std::time::Duration::from_millis(250);
    //let path = match env::args().nth(1) {
    //    Some(arg) => { arg },
    //    None => { 
    //        println!("There is no option.");
    //        return;
    //    }
    //};

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

    let mut kc = KeyConverter::new();
    // 最後にpushしたキーコードを入れておく
    let mut last_push = 0;

    //loop {
    for _ in 0..50 {
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

        // 本来、statusの値はコマンドのオプションで決めるようにする
        let show_state = false;
        #[cfg(debug_assertions)]
        let show_state = true;

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

    /*
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
    */
