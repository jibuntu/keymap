#![allow(dead_code)]
/// キーを変換するためのモジュール

use std::collections::HashSet;
use std::collections::HashMap;
use std::io::Read;

mod rules;
use self::rules::Rules;
use self::rules::Key;


pub struct KeyConverter {
    keys: HashSet<Key>, // 実際に押されているキーのリスト
    vkeys: Vec<Key>, // 仮想的に押されているキーのリスト
    rules_list: HashMap<String, Box<Rules>>,
    rules: Option<Box<Rules>>,
}

impl KeyConverter {
    pub fn new<R: Read>(r: R) -> Result<KeyConverter, String> {
        let rules_list = match Rules::new(r) {
            Ok(rules) => rules,
            Err(e) => return Err(e)
        };

        let mut rules_list: HashMap<String, Box<Rules>> = 
             rules_list.into_iter().map(|(s, r)| (s, Box::new(r))).collect();
        let rules = rules_list.remove("").unwrap();

        Ok(KeyConverter {
            keys: HashSet::new(),
            vkeys: Vec::new(),
            rules_list,
            rules: Some(rules)
        })
    }
    
    /// 前回とのvkeysの差分を元に返り値を返す。
    /// 返り値は押すキーと離すキー
    pub fn push(&mut self, k: u16) -> (Vec<u16>, Vec<u16>) {
        self.keys.insert(Key::Raw(k));
        let vk = self.rules.as_ref().unwrap().filter(&self.keys);

        // ルールを変える場合は何も押さずに全て離す
        for v in &vk {
            let name = if let Key::Rule(name) = v {
                name
            } else {
                continue;
            };

            //// 新しいルールをself.rulesに入れる
            let new = self.rules_list.remove(name).unwrap();
            let old = self.rules.replace(new).unwrap();

            //// 以前のルールをself.rules_listに入れる
            self.rules_list.insert(old.get_name().to_string(), old);
        
            // 何も押さずに、すべてのキーを離す
            let vkeys = self.vkeys.iter()
                .map(|v| v.to_u16().unwrap()).collect();
            self.vkeys.clear();

            // 実際に押されているキーもすべてなかったことにする
            self.keys.clear();

            return (Vec::new(), vkeys)
        }

        // vk - vkeys の結果のキーを押す
        let push = vk.iter()
             .filter(|k| !self.vkeys.contains(&k))
             .map(|k| k.to_u16().unwrap()).collect();

        // vkeysに入っていて、vkに入っていないキーを離す
        // vkeys - vk の結果のキーを離す
        let leave = self.vkeys.iter()
             .filter(|k| !vk.contains(&k))
             .map(|k| k.to_u16().unwrap()).collect();

        // self.vkeysの値を更新する
        self.vkeys = vk;

        (push, leave)
    }

    /// 前回とのvkeysの差分を元に返り値を返す。
    /// ここではState::Pushとなるキーは返さない。
    /// 返り値は離すキーのリスト
    pub fn leave(&mut self, k: u16) -> Vec<u16> { 
        self.keys.remove(&Key::Raw(k));
        let vk = self.rules.as_ref().unwrap().filter(&self.keys);
        
        // vkeysに入っていて、vkに入っていないキーを離す
        // vkeys - vk の結果のキーを離す
        let leave = self.vkeys.iter()
             .filter(|k| !vk.contains(&k))
             .map(|k| k.to_u16().unwrap()).collect();

        // self.vkeysの値を更新する
        self.vkeys = vk;

        leave
    }

    pub fn filter_to_string(&mut self) -> String {
        self.rules.as_ref().unwrap().filter_to_string(&self.keys)
    }
}

