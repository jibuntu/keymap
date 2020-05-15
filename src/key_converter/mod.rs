#![allow(dead_code)]
/// キーを変換するためのモジュール

use std::collections::HashSet;
use std::io::Read;

mod rules;
use self::rules::Rules;
use self::rules::Key;


pub struct KeyConverter {
    keys: HashSet<Key>, // 実際に押されているキーのリスト
    vkeys: Vec<Key>, // 仮想的に押されているキーのリスト
    rules: Rules,
}

impl KeyConverter {
    pub fn new<R: Read>(r: R) -> Result<KeyConverter, String> {
        let rules = match Rules::new(r) {
            Ok(rules) => rules,
            Err(e) => return Err(e)
        };

        Ok(KeyConverter {
            keys: HashSet::new(),
            vkeys: Vec::new(),
            rules
        })
    }
    
    /// 前回とのvkeysの差分を元に返り値を返す。
    /// 返り値は押すキーと離すキー
    pub fn push(&mut self, k: u16) -> (Vec<u16>, Vec<u16>) {
        self.keys.insert(Key::Raw(k));
        let vk = self.rules.filter(&self.keys);

        // vk - vkeys の結果のキーを押す
        let push = vk.iter()
             .filter(|k| !self.vkeys.contains(&k))
             .map(|k| k.to_u16()).collect();

        // vkeysに入っていて、vkに入っていないキーを離す
        // vkeys - vk の結果のキーを離す
        let leave = self.vkeys.iter()
             .filter(|k| !vk.contains(&k))
             .map(|k| k.to_u16()).collect();

        // self.vkeysの値を更新する
        self.vkeys = vk;

        (push, leave)
    }

    /// 前回とのvkeysの差分を元に返り値を返す。
    /// ここではState::Pushとなるキーは返さない。
    /// 返り値は離すキーのリスト
    pub fn leave(&mut self, k: u16) -> Vec<u16> { 
        self.keys.remove(&Key::Raw(k));
        let vk = self.rules.filter(&self.keys);
        
        // vkeysに入っていて、vkに入っていないキーを離す
        // vkeys - vk の結果のキーを離す
        let leave = self.vkeys.iter()
             .filter(|k| !vk.contains(&k))
             .map(|k| k.to_u16()).collect();

        // self.vkeysの値を更新する
        self.vkeys = vk;

        leave
    }

    pub fn filter_to_string(&mut self) -> String {
        self.rules.filter_to_string(&self.keys)
    }
}

