use std::cmp::Eq;
use std::hash::Hash;

pub mod keycode;
use self::keycode::Keycode;


lazy_static! {
    // 毎回作成するのは非効率なのでstaticにしておく
    static ref KEYCODE: Keycode = Keycode::new();
}

/// 変換される前の元々のキーと、変換された後のキーを区別する
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Raw(u16), // 変換される前のキー
    Con(u16), // 変換された後のキー (convert)
    Rule(String) // ルール名
}

impl Key {
    /// 文字列からKeyを作成する
    pub fn from_str(s: &str) -> Result<Key, String> {
        match s.chars().next() {
            Some('\'') => match s.get(1..) {
                Some(s) => match KEYCODE.from_keyword(s) {
                    Some(k) => return Ok(Key::Con(k)),
                    None => {
                        return Err(format!("'{}'は無効なキーコードです", s))
                    }
                },
                None => return Err(format!("'{}'は無効なキーコードです", s))
            },
            Some('@') => match s.get(1..) {
                Some(s) => Ok(Key::Rule(s.to_string())),
                None => Ok(Key::Rule(String::new())),
            },
            Some(_) => match KEYCODE.from_keyword(s) {
                Some(k) => return Ok(Key::Raw(k)),
                None => return Err(format!("'{}'は無効なキーコードです", s))
            },
            None => return Err(format!("'{}'は無効なキーコードです", s))
        }
    }

    pub fn to_u16(&self) -> Option<u16> {
        match self {
            Key::Raw(n) => Some(*n),
            Key::Con(n) => Some(*n),
            Key::Rule(_) => None
        }
    }

    /// Keycodeモジュールを使いキーコードを文字列に変換する
    pub fn to_string(&self) -> String {
        let s = match self {
            Key::Raw(n) => {
                KEYCODE.from_keycode(*n).unwrap_or("UNKNOWN".to_string())
            },
            Key::Con(n) => {
                "'".to_string() + &KEYCODE.from_keycode(*n)
                                          .unwrap_or("UNKNOWN".to_string())
            },
            Key::Rule(s) => s.clone()
        };

        s
    }
}

#[cfg(test)]
mod test_key {
    use super::Keycode;
    use super::Key;

    #[test]
    fn test_key_from_str() {
        let keycode = Keycode::new();

        let mut s = "";
        assert_eq!(Key::from_str(& mut s), Err(format!("'{}'は無効なキーコードです", "")));

        let mut s = "  ";
        assert_eq!(Key::from_str(& mut s), Err(format!("'{}'は無効なキーコードです", "  ")));

        let mut s = "A";
        assert_eq!(Key::from_str(& mut s), Ok(Key::Raw(keycode.from_keyword("A").unwrap())));

        let mut s = "'B";
        assert_eq!(Key::from_str(& mut s), Ok(Key::Con(keycode.from_keyword("B").unwrap())));

        let mut s = "RIGHTALT";
        assert_eq!(Key::from_str(& mut s), Ok(Key::Raw(keycode.from_keyword("RIGHTALT").unwrap())));

        let mut s = "'RIGHTSHIFT";
        assert_eq!(Key::from_str(& mut s), Ok(Key::Con(keycode.from_keyword("RIGHTSHIFT").unwrap())));
    }
}


/// KeyRule
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRule {
    pub k: Vec<Key>,
    pub v: Vec<Key>,
    pub ove: bool
}

impl KeyRule {
    pub fn new(k: Vec<Key>, v: Vec<Key>) -> KeyRule {
        KeyRule {
            k: k,
            v: v,
            ove: false
        }
    }

    pub fn with_ove(k: Vec<Key>, v: Vec<Key>, ove: bool) -> KeyRule {
        KeyRule {
            k: k,
            v: v,
            ove
        }
    }

    /// self.kの値が引数と同じかどうかを比較する
    pub fn compare_k(&self, k: &Vec<Key>) -> bool {
        if self.k.len() != k.len() {
            return false
        }

        self.k.iter().zip(k.iter()).all(|(a, b)| a == b)
    }

    // 文字列からKeyRuleを作成する
    // boolは上書きするかどうか
    pub fn from_str(string: &str) -> Result<KeyRule, String> {
        let mut klist = Vec::new();
        let mut vlist = Vec::new();
        let mut s;
        let mut ove = false;

        if string.contains("->") {
            s = string.split("->");
        } else if string.contains("-!>") {
            s = string.split("-!>");
            ove = true;
        } else {
            return Err(format!("'->' or '-!>' がありません"))
        }

        let kstr = match s.next() {
            Some(kstr) => kstr,
            None => return Err(format!("左側の値がありません"))
        };
        let vstr = match s.next() {
            Some(vstr) => vstr,
            None => return Err(format!("右側の値がありません"))
        };

        for k in kstr.split("+").map(|k| k.trim()) {
            match Key::from_str(k) {
                Ok(k) => {
                    klist.push(k);
                },
                Err(e) => return Err(e)
            }
        }

        for v in vstr.split("+").map(|v| v.trim()) {
            match Key::from_str(v) {
                Ok(v) => {
                    vlist.push(v);
                },
                Err(e) => return Err(e)
            }
        }

        if klist.len() == 0 || vlist.len() == 0 {
            return Err(format!("左側または右側の値がありません"))
        }

        Ok(KeyRule {
            k: klist,
            v: vlist,
            ove
        })
    }

    /// ルールを文字列へ変換する
    pub fn to_string(&self) -> String {
        let mut s = String::new();

        for (i, k) in self.k.iter().enumerate() {
            s += &k.to_string();
            if i != self.k.len()-1 {
                s += " + ";
            } else {
                s += " -> ";
            }
        }

        for (i, v) in self.v.iter().enumerate() {
            s += &v.to_string();
            if i != self.v.len()-1 {
                s += " + ";
            }
        }

        s
    }
}