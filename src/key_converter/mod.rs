#![allow(dead_code)]
/// キーを変換するためのモジュール

use std::collections::HashSet;

mod rules;
use self::rules::Rules;
use self::rules::Key;


#[derive(Debug, PartialEq)]
pub enum KeyState {
    Push(u16),
    Repeat(u16),
    Leave(u16)
}

pub struct KeyConverter {
//    keys: HashSet<Key>, // 実際に押されているキーのリスト
//    vkeys: HashSet<Key>, // 仮想的に押されているキーのリスト
//    rule: Rules,
}

impl KeyConverter {
    /*pub fn with_rule(rule: Rules) -> KeyConverter {
        KeyConverter {
            keys: HashSet::new(),
            vkeys: HashSet::new(),
            rule
        }
    }
    pub fn push(&mut self, key: Key) -> Vec<KeyState> { Vec::new() }
    pub fn leave(&mut self, key: Key) -> Vec<KeyState> { Vec::new() }
    */
}

