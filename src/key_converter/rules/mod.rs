#![allow(dead_code)]
/// ファイルからルールのリストを作る。
/// キーのリストを受け取り、ルールに合うように変換する。

use std::collections::HashSet;
use std::iter::FromIterator;
use std::cmp::Eq;
use std::hash::Hash;

pub mod keycode;
use self::keycode::Keycode;


/// 変換される前の元々のキーと、変換された後のキーを区別する
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Raw(u16), // 変換される前のキー
    Con(u16) // 変換された後のキー (convert)
}

impl Key {
    pub fn to_u16(&self) -> u16 {
        match self {
            Key::Raw(n) => *n,
            Key::Con(n) => *n
        }
    }
}

/// KeyRule
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRule {
    k: HashSet<Key>,
    v: HashSet<Key>,
}

impl KeyRule {
    pub fn new(k: Vec<Key>, v: Vec<Key>) -> KeyRule {
        KeyRule {
            k: HashSet::from_iter(k.into_iter()),
            v: HashSet::from_iter(v.into_iter())
        }
    }
}

/// ルールの構造体
pub struct Rules {
    list: Vec<KeyRule>,
}

impl Rules {
    pub fn new() -> Rules {
        Rules {
            list: Vec::new(),
        }
    }

    pub fn from_vec(v: Vec<KeyRule>) -> Rules {
        Rules {
            list: v
        }
    }

    /// 再帰的に実行する関数。
    /// 第一引数のkeysは現在押されているキーのリスト（仮想的なキーも含む）。
    /// 第二引数のmatched_rulesはすでにマッチしているルールへの参照。同じルール
    /// に複数回マッチしないようにするため。
    fn filter_recursion<'a>(&'a self, 
                            keys: &mut HashSet<Key>, 
                            matched_rules: &mut Vec<&'a KeyRule>) 
    {
        for key_rule in &self.list {
            // matched_rulesに入っているルールを除外した上でサブセットかどうか
            if !matched_rules.contains(&key_rule) && key_rule.k.is_subset(&keys) {
                // println!("MATCH: {:?} -> {:?}", key_rule.k, key_rule.v);

                // マッチしたルールをmatched_rulesに追加する
                matched_rules.push(key_rule);

                // マッチしたルールの値をすべてkeysに入れる
                for v in &key_rule.v {
                    keys.insert(v.clone());
                }

                // 新しくkeysをセットしたので、それをもとに再帰的に呼び出す
                self.filter_recursion(keys, matched_rules);

                // いずれかにマッチした時点でループをやめる
                break
            }
        }
    }
    
    // ルールを元に引数のKeysをvkeysに変換する
    pub fn filter(&self, keys: &HashSet<Key>) -> HashSet<Key> {
        let mut matched_rules = Vec::new();
        // vkeysの初期値は、keysの値
        let mut vkeys = keys.clone();

        // vkeysとmatched_rulesの値をセットする
        self.filter_recursion(&mut vkeys, &mut matched_rules);

        // ルールのキーとして使われているのにvkeysに入っているキーを削除する
        // また、ルールのキーにマッチしなかったキーはそのまま残る
        for k in matched_rules.iter().map(|r| &r.k).flatten() {
            vkeys.remove(k);
        }

        vkeys
    }
}


#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use super::Key;
    use super::KeyRule;
    use super::Rules;
    use super::Keycode;

    macro_rules! hash {
        ($($x:expr),*) => {
            {
              let mut temp_set = HashSet::new();
              $(temp_set.insert($x);)*
              temp_set
            }
        };
    }

    #[test]
    fn test_rule_filter() {
        #![allow(non_snake_case)]
        #![allow(unused_variables)]
        let code = Keycode::new();

        // keycodeモジュールからキーコードを取得
        let KEY_A: u16 = code.from_keyword("A").unwrap();
        let KEY_B: u16 = code.from_keyword("B").unwrap();
        let KEY_C: u16 = code.from_keyword("C").unwrap();
        let KEY_D: u16 = code.from_keyword("D").unwrap();
        let KEY_E: u16 = code.from_keyword("E").unwrap();
        let KEY_F: u16 = code.from_keyword("F").unwrap();
        let KEY_G: u16 = code.from_keyword("G").unwrap();
        let KEY_H: u16 = code.from_keyword("H").unwrap();
        let KEY_I: u16 = code.from_keyword("I").unwrap();
        let KEY_J: u16 = code.from_keyword("J").unwrap();
        let KEY_CTRL: u16 = code.from_keyword("LEFTCTRL").unwrap();
        let KEY_SHIFT: u16 = code.from_keyword("LEFTSHIFT").unwrap();
        let KEY_ENTER: u16 = code.from_keyword("ENTER").unwrap();
        let KEY_ALT: u16 = code.from_keyword("LEFTALT").unwrap();
        let KEY_BACKSPACE: u16 = code.from_keyword("BACKSPACE").unwrap();
        let KEY_TAB: u16 = code.from_keyword("TAB").unwrap();
        let KEY_LEFT: u16 = code.from_keyword("LEFT").unwrap();
        let KEY_RIGHT: u16 = code.from_keyword("RIGHT").unwrap();
        let KEY_UP: u16 = code.from_keyword("UP").unwrap();
        let KEY_DOWN: u16 = code.from_keyword("DOWN").unwrap();

        let rule = Rules::new();
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_A)]), hash![Key::Raw(KEY_A)]);

        let mut rule = Rules::new();
        // A -> 'H
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_A)], vec![Key::Con(KEY_H)]));
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_A)]), hash![Key::Con(KEY_H)]);

        let mut rule = Rules::new();
        // A -> 'H
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_A)], vec![Key::Con(KEY_H)]));
        // 'H -> 'A
        rule.list.push(KeyRule::new(vec![Key::Con(KEY_H)], vec![Key::Con(KEY_A)]));
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_A)]), hash![Key::Con(KEY_A)]);


        let mut rule = Rules::new();
        // B -> 'I 
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_B)], vec![Key::Con(KEY_I)]));
        // I -> 'B 
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_I)], vec![Key::Con(KEY_B)]));
        // A -> 'C
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_A)], vec![Key::Con(KEY_C)]));
        // 'B + 'C -> 'ENTER
        rule.list.push(KeyRule::new(vec![Key::Con(KEY_B), Key::Con(KEY_C)], vec![Key::Con(KEY_ENTER)]));

        // push B
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_B)]), hash![Key::Con(KEY_I)]);
        // push I
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_I)]), hash![Key::Con(KEY_B)]);
        // push A
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_A)]), hash![Key::Con(KEY_C)]);
        // push B + I 
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_B), Key::Raw(KEY_I)]), hash![Key::Con(KEY_I), Key::Con(KEY_B)]);
        // push I + A // ここでは'B + 'Cに変換され、その後'ENTREに変換される
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_I), Key::Raw(KEY_A)]), hash![Key::Con(KEY_ENTER)]);
        // push I + A + B
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_I), Key::Raw(KEY_A), Key::Raw(KEY_B)]), hash![Key::Con(KEY_ENTER), Key::Con(KEY_I)]);

        let mut rule = Rules::new();
        // B -> 'I 
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_B)], vec![Key::Con(KEY_I)]));
        // ALT -> 'CTRL 
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_ALT)], vec![Key::Con(KEY_CTRL)]));
        // 'I + 'CTRL -> 'ENTER
        rule.list.push(KeyRule::new(vec![Key::Con(KEY_I), Key::Con(KEY_CTRL)], vec![Key::Con(KEY_ENTER)]));
        // C + 'J
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_C)], vec![Key::Con(KEY_J)]));
        // 'J + 'CTRL -> 'SHIFT
        rule.list.push(KeyRule::new(vec![Key::Con(KEY_J), Key::Con(KEY_CTRL)], vec![Key::Con(KEY_SHIFT)]));
        // push B + ALT = 'ENTER
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_B), Key::Raw(KEY_ALT)]), hash![Key::Con(KEY_ENTER)]);
        // push C + ALT = 'SHIFT
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_C), Key::Raw(KEY_ALT)]), hash![Key::Con(KEY_SHIFT)]);
        // push B + C + ALT = 'ENTER + 'SHIFT
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_B), Key::Raw(KEY_C), Key::Raw(KEY_ALT)]), hash![Key::Con(KEY_ENTER), Key::Con(KEY_SHIFT)]);

        // 'ENTER + 'SHIFT -> 'A
        rule.list.push(KeyRule::new(vec![Key::Con(KEY_ENTER), Key::Con(KEY_SHIFT)], vec![Key::Con(KEY_A)]));
        // push B + C + ALT = 'A
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_B), Key::Raw(KEY_C), Key::Raw(KEY_ALT)]), hash![Key::Con(KEY_A)]);

        // 'ENTER + 'SHIFT + BACKSPACE + TAB -> 'LEFT + 'RIGHT + 'UP + 'DOWN
        rule.list.push(
            KeyRule::new(
                vec![Key::Con(KEY_ENTER), Key::Con(KEY_SHIFT), Key::Raw(KEY_BACKSPACE), Key::Raw(KEY_TAB)], 
                vec![Key::Con(KEY_LEFT), Key::Con(KEY_RIGHT), Key::Con(KEY_UP), Key::Con(KEY_DOWN)]
            )
        );
        // push B + C + ALT + BACKSPACE + TAB = 'A + 'LEFT + 'RIGHT + 'UP + 'DOWN
        // B + C + ALTを変換した後の'ENTER + 'SHIFTは'Aになり、
        // その上で今回のルールのキーにもなっている
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_B), Key::Raw(KEY_C), Key::Raw(KEY_ALT), Key::Raw(KEY_BACKSPACE), Key::Raw(KEY_TAB)]), hash![Key::Con(KEY_A), Key::Con(KEY_LEFT), Key::Con(KEY_RIGHT), Key::Con(KEY_UP), Key::Con(KEY_DOWN)]);

        // 'A + 'LEFT + 'RIGHT + 'UP + 'DOWN -> 'F
        rule.list.push(
            KeyRule::new(
                vec![Key::Con(KEY_A), Key::Con(KEY_LEFT), Key::Con(KEY_RIGHT), Key::Con(KEY_UP), Key::Con(KEY_DOWN)],
                vec![Key::Con(KEY_F)]
            )
        );
        // push B + C + ALT + BACKSPACE + TAB = 'F
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_B), Key::Raw(KEY_C), Key::Raw(KEY_ALT), Key::Raw(KEY_BACKSPACE), Key::Raw(KEY_TAB)]), hash![Key::Con(KEY_F)]);
    }
}