#![allow(dead_code)]
/// ファイルからルールのリストを作る。
/// キーのリストを受け取り、ルールに合うように変換する。

use std::collections::HashSet;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::io::Read;

pub mod key_rule;
use self::key_rule::KeyRule;
use self::key_rule::Key;
use self::key_rule::keycode::Keycode;


lazy_static! {
    // 毎回作成するのは非効率なのでstaticにしておく
    static ref KEYCODE: Keycode = Keycode::new();
}

/// ルールの構造体
#[derive(Debug, Clone, PartialEq)]
pub struct Rules {
    name: String,
    extend: Option<String>, // 継承するルール名
    list: Vec<KeyRule>,
}

/// 再帰的に継承をする
/// 引数のnameのルールの継承を適用する
fn attach_ex_recursion(rules_list: &mut HashMap<String, Rules>, 
                       extended_names: &mut HashSet<String>, 
                       name_history: &mut HashSet<String>,
                       name: String) -> Result<(), String>
{
    let r = match rules_list.get(&name) {
        Some(r) => r,
        None => return Err(format!("'{}'というルールは存在しません", name))
    };

    // 継承先がなければ何もしない
    if let None = r.extend {
        return Ok(())
    }

    // 継承が循環していたらエラーを返す
    if name_history.contains(&name) {
        return Err(format!("'@{}' 継承が循環しています", name))
    }
    name_history.insert(name.clone());
    
    let e = r.extend.clone().unwrap();

    // 継承先がまだ継承を適用されていなければ継承をする
    if !extended_names.contains(&e) {
        attach_ex_recursion(
            rules_list, extended_names, name_history, e.clone())?;
    }

    let ex_list = match rules_list.get(&e) {
        Some(r) => r.list.clone(),
        None => return Err(format!("'{}'というルールは存在しません", name))
    };

    // rはすでにattach_ex_recursionでmutable borrowしているので、再取得する
    let r = match rules_list.get_mut(&name) {
        Some(r) => r,
        None => return Err(format!("'{}'というルールは存在しません", name))
    };

    // リストを結合する
    for r_ex in ex_list {
        r.list.push(r_ex);
    }

    // 結合したルールを追加する
    extended_names.insert(name.to_string());
    Ok(())
}

/// 継承を適用する
fn attach_ex(rules_list: &mut HashMap<String, Rules>) -> Result<(), String> {
    // 継承済みのルール名を入れておく
    let mut extended_names = HashSet::new();
    let name_list: Vec<String> = rules_list.iter().map(|(n, _)| n.clone()).collect();

    // rules_listの継承を適用する
    for name in name_list {
        // 循環しないように継承元の名前を持っておく
        let mut name_history = HashSet::new();

        attach_ex_recursion(rules_list, 
                            &mut extended_names, 
                            &mut name_history, 
                            name.to_string())?;
    }

    Ok(())
}

impl Rules {
    /// ストリームからRulesを作る
    pub fn new<R: Read>(mut r: R) -> Result<HashMap<String, Rules>, String> {
        let mut s = String::new();
        let mut list = Vec::new();
        let mut extend = None;
        let mut rule_name = String::new();
        let mut rules_list = HashMap::new();

        let _ = r.read_to_string(&mut s);

        // コメントを削除して、不要な行を削除する
        let lines = s.lines()
            .map(|l| l.split('#').next().unwrap())
            .map(|l| l.trim())
            .enumerate()
            .filter(|(_, l)| l.len() != 0);

        //for (i, l) in lines {
        for (i, l) in lines {
            match l.chars().next().unwrap() {
                // '@'が来たらルールを作成して、rules_listに追加する
                '@' => match l.get(1..) {
                    Some(n) => {
                        let r = Rules::from_vec(
                            &rule_name, extend, list.clone());

                        rules_list.insert(rule_name, r);

                        // listの要素を初期化
                        list.clear();

                        // rule_nameを新しいものにする
                        rule_name = 
                            n.split(':').next().unwrap().trim().to_string();

                        // extendを新しいものにする
                        if let Some(ex) = n.split(':').nth(1) {
                            match ex.trim().get(1..) {
                                Some(ex) => {
                                    extend = Some(ex.to_string());
                                }
                                // Noneのときはルール名は空文字列
                                None => {
                                    extend = Some(String::new());
                                }
                            }
                        } else {
                            extend = None;
                        }
                    },
                    None => return Err(format!("ルール名がありません"))
                },
                _ => match KeyRule::from_str(l) {
                    Ok(k) => list.push(k),
                    Err(e) => return Err(format!("{}: line {}", e, i+1))
                }
            }
        }

        // 最後にrules_listに追加する
        let r = Rules::from_vec(&rule_name, extend, list.clone());
        rules_list.insert(rule_name, r);

        // 継承を適用する
        attach_ex(&mut rules_list)?;

        Ok(rules_list)
    }

    pub fn from_vec(name: &str, 
                    extend: Option<String>, 
                    v: Vec<KeyRule>) -> Rules 
        {
        Rules {
            name: name.to_string(),
            extend,
            list: v
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// 再帰的に実行する関数。
    /// 第一引数のkeysは現在押されているキーのリスト（仮想的なキーも含む）。
    /// 第二引数のmatched_rulesはすでにマッチしているルールへの参照。同じルール
    /// に複数回マッチしないようにするため。
    fn filter_recursion<'a>(&'a self, 
                            keys: &mut Vec<Key>, 
                            matched_rules: &mut Vec<&'a KeyRule>) 
    {
        'outer: for key_rule in &self.list {
            // matched_rulesに入っているルールを除外した上でサブセットかどうか
            if matched_rules.contains(&key_rule) {
                continue;
            }
            
            for k in &key_rule.k {
                if !keys.contains(&k) {
                    continue 'outer;
                }
            }

            // println!("MATCH: {:?} -> {:?}", key_rule.k, key_rule.v);

            // マッチしたルールをmatched_rulesに追加する
            matched_rules.push(key_rule);

            // マッチしたルールの値をすべてkeysに入れる
            for v in &key_rule.v {
                // すでにあるものは除外する
                if !keys.contains(&v) {
                    keys.push(v.clone());
                }
            }

            // 新しくkeysをセットしたので、それをもとに再帰的に呼び出す
            self.filter_recursion(keys, matched_rules);

            // いずれかにマッチした時点でループをやめる
            break
        }
    }
    
    // ルールを元に引数のKeysをvkeysに変換する
    pub fn filter(&self, keys: &HashSet<Key>) -> Vec<Key> {
        let mut matched_rules = Vec::new();
        // vkeysの初期値は、keysの値
        let mut vkeys: Vec<Key> = Vec::from_iter(keys.iter().map(|k| k.clone()));
        let mut result: Vec<Key> = Vec::new();

        // vkeysとmatched_rulesの値をセットする
        self.filter_recursion(&mut vkeys, &mut matched_rules);
        let keys: Vec<Key> = matched_rules.iter().map(|r| r.k.clone()).flatten().collect();

        // ルールのキーとして使われているのにvkeysに入っているキーを削除する
        // また、ルールのキーにマッチしなかったキーはそのまま残る
        for vk in vkeys {
            if !keys.contains(&vk) {
                result.push(vk.clone());
            }
        }
        
        result
    }

    // ルールを元に引数のKeysをvkeysに変換し、それを文字列にする
    pub fn filter_to_string(&self, keys: &HashSet<Key>) -> String {
        // 最初はfilter関数と同じ処理
        let mut matched_rules = Vec::new();
        let mut vkeys: Vec<Key> = Vec::from_iter(keys.iter().map(|k| k.clone()));
        let mut result: Vec<Key> = Vec::new();

        self.filter_recursion(&mut vkeys, &mut matched_rules);
        let keys: Vec<Key> = matched_rules.iter().map(|r| r.k.clone()).flatten().collect();

        for vk in &vkeys {
            if !keys.contains(&vk) {
                result.push(vk.clone());
            }
        }

        // ここから文字列へ変換してゆく
        let mut s = String::new();

        //s += &matched_rules.iter().map(|r| r.to_string()).collect::<String>();

        for (i, r) in matched_rules.iter().enumerate() {
            s += &r.to_string();
            if i != matched_rules.len()-1 {
                s += " , ";
            } else {
                s += "  :  ";
            }
        }

        for (i, k) in result.iter().enumerate() {
            s += &k.to_string();
            if i != result.len()-1 {
                s += " + ";
            }
        }
        
        s
    }
}


#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::collections::HashMap;
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
    fn test_rule_new() {
        let code = Keycode::new();

        let r = Rules::new("".as_bytes()).unwrap().remove("").unwrap();
        assert_eq!(r.list, vec![]);

        if let Ok(_) = Rules::new("->".as_bytes()) { panic!() }
        if let Ok(_) = Rules::new("a->".as_bytes()) { panic!() } 
        if let Ok(_) = Rules::new("->mm".as_bytes()) { panic!() } 

        let r = Rules::new("A -> B".as_bytes()).unwrap().remove("").unwrap();
        assert_eq!(r.list, vec![
            KeyRule::new(vec![Key::Raw(code.from_keyword("A").unwrap())], vec![Key::Raw(code.from_keyword("B").unwrap())])
        ]);

        let mut r = Rules::new(r#"
        # test
        A -> 'B
        B -> 'A # test
        # aiueo
        C -> 'ENTER
        "#.as_bytes()).unwrap();
        assert_eq!(r.remove("").unwrap().list, vec![
            KeyRule::new(vec![Key::Raw(code.from_keyword("A").unwrap())], vec![Key::Con(code.from_keyword("B").unwrap())]),
            KeyRule::new(vec![Key::Raw(code.from_keyword("B").unwrap())], vec![Key::Con(code.from_keyword("A").unwrap())]),
            KeyRule::new(vec![Key::Raw(code.from_keyword("C").unwrap())], vec![Key::Con(code.from_keyword("ENTER").unwrap())]),
        ]);

        let r = Rules::new(r#"
        A -> 'B
        B -> 'A
        "#.as_bytes()).unwrap();
        let mut rlist = HashMap::new();
        rlist.insert("".to_string(), Rules { name: String::new(), extend: None, list: vec![
            KeyRule::new(vec![Key::Raw(code.from_keyword("A").unwrap())], vec![Key::Con(code.from_keyword("B").unwrap())]),
            KeyRule::new(vec![Key::Raw(code.from_keyword("B").unwrap())], vec![Key::Con(code.from_keyword("A").unwrap())]),
        ]});
        assert_eq!(r, rlist);

        let r = Rules::new(r#"
        A -> 'B
        B -> 'A
        @RULE1
            M -> 'N
            N -> 'M
        
        @RULE2
            X + Y -> 'Z
            ENTER -> @RULE1
        "#.as_bytes()).unwrap();
        let mut rlist = HashMap::new();
        rlist.insert("".to_string(), Rules { name: String::new(), extend: None, list: vec![
            KeyRule::new(vec![Key::Raw(code.from_keyword("A").unwrap())], vec![Key::Con(code.from_keyword("B").unwrap())]),
            KeyRule::new(vec![Key::Raw(code.from_keyword("B").unwrap())], vec![Key::Con(code.from_keyword("A").unwrap())]),
        ]});
        rlist.insert("RULE1".to_string(), Rules { name: "RULE1".to_string(), extend: None, list: vec![
            KeyRule::new(vec![Key::Raw(code.from_keyword("M").unwrap())], vec![Key::Con(code.from_keyword("N").unwrap())]),
            KeyRule::new(vec![Key::Raw(code.from_keyword("N").unwrap())], vec![Key::Con(code.from_keyword("M").unwrap())]),
        ]});
        rlist.insert("RULE2".to_string(), Rules { name: "RULE2".to_string(), extend: None, list: vec![
            KeyRule::new(vec![Key::Raw(code.from_keyword("X").unwrap()), Key::Raw(code.from_keyword("Y").unwrap())], vec![Key::Con(code.from_keyword("Z").unwrap())]),
            KeyRule::new(vec![Key::Raw(code.from_keyword("ENTER").unwrap())], vec![Key::Rule("RULE1".to_string())]),
        ]});
        assert_eq!(r, rlist);

        /* 継承のテスト */
        let r = Rules::new(r#"
        @RULE1
        @RULE2 : @RULE1
        @RULE3 : @
        "#.as_bytes()).unwrap();
        let mut rlist = HashMap::new();
        rlist.insert("".to_string(), Rules { name: String::new(), extend: None, list: vec![]});
        rlist.insert("RULE1".to_string(), Rules { name: "RULE1".to_string(), extend: None, list: vec![]});
        rlist.insert("RULE2".to_string(), Rules { name: "RULE2".to_string(), extend: Some("RULE1".to_string()), list: vec![]});
        rlist.insert("RULE3".to_string(), Rules { name: "RULE3".to_string(), extend: Some("".to_string()), list: vec![]});
        assert_eq!(r, rlist);

        let r = Rules::new(r#"
        @RULE1
            A -> 'B
            B -> 'A
        @RULE2 : @RULE1
            M -> 'N
            N -> 'M
        "#.as_bytes()).unwrap();
        let mut rlist = HashMap::new();
        rlist.insert("".to_string(), Rules { name: String::new(), extend: None, list: vec![]});
        rlist.insert("RULE1".to_string(), Rules { name: "RULE1".to_string(), extend: None, list: vec![
            KeyRule::new(vec![Key::Raw(code.from_keyword("A").unwrap())], vec![Key::Con(code.from_keyword("B").unwrap())]),
            KeyRule::new(vec![Key::Raw(code.from_keyword("B").unwrap())], vec![Key::Con(code.from_keyword("A").unwrap())]),
        ]});
        rlist.insert("RULE2".to_string(), Rules { name: "RULE2".to_string(), extend: Some("RULE1".to_string()), list: vec![
            KeyRule::new(vec![Key::Raw(code.from_keyword("M").unwrap())], vec![Key::Con(code.from_keyword("N").unwrap())]),
            KeyRule::new(vec![Key::Raw(code.from_keyword("N").unwrap())], vec![Key::Con(code.from_keyword("M").unwrap())]),
            KeyRule::new(vec![Key::Raw(code.from_keyword("A").unwrap())], vec![Key::Con(code.from_keyword("B").unwrap())]),
            KeyRule::new(vec![Key::Raw(code.from_keyword("B").unwrap())], vec![Key::Con(code.from_keyword("A").unwrap())]),
        ]});
        assert_eq!(r, rlist);

        let r = r#"
        @RULE1 : @RULE2
        @RULE2 : @RULE3
        @RULE3 : @RULE1
        "#.as_bytes();
        // RULE1, RULE2, RULE3のいずれか
        match Rules::new(r) {
            Err(e) if &e == "'@RULE1' 継承が循環しています" => (),
            Err(e) if &e == "'@RULE2' 継承が循環しています" => (),
            Err(e) if &e == "'@RULE3' 継承が循環しています" => (),
            _ => panic!()
        }
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

        let rule = Rules::new("".as_bytes()).unwrap().remove("").unwrap();
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_A)]), vec![Key::Raw(KEY_A)]);


        let mut rule = Rules::new("".as_bytes()).unwrap().remove("").unwrap();
        // A -> 'H
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_A)], vec![Key::Con(KEY_H)]));
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_A)]), vec![Key::Con(KEY_H)]);

        let mut rule = Rules::new("".as_bytes()).unwrap().remove("").unwrap();
        // A -> 'H
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_A)], vec![Key::Con(KEY_H)]));
        // 'H -> 'A
        rule.list.push(KeyRule::new(vec![Key::Con(KEY_H)], vec![Key::Con(KEY_A)]));
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_A)]), vec![Key::Con(KEY_A)]);

        let mut rule = Rules::new("".as_bytes()).unwrap().remove("").unwrap();
        // B -> 'I 
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_B)], vec![Key::Con(KEY_I)]));
        // I -> 'B 
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_I)], vec![Key::Con(KEY_B)]));
        // A -> 'C
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_A)], vec![Key::Con(KEY_C)]));
        // 'B + 'C -> 'ENTER
        rule.list.push(KeyRule::new(vec![Key::Con(KEY_B), Key::Con(KEY_C)], vec![Key::Con(KEY_ENTER)]));

        // push B
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_B)]), vec![Key::Con(KEY_I)]);
        // push I
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_I)]), vec![Key::Con(KEY_B)]);
        // push A
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_A)]), vec![Key::Con(KEY_C)]);
        // push B + I 
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_B), Key::Raw(KEY_I)]), vec![Key::Con(KEY_I), Key::Con(KEY_B)]);
        // push I + A // ここでは'B + 'Cに変換され、その後'ENTREに変換される
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_I), Key::Raw(KEY_A)]), vec![Key::Con(KEY_ENTER)]);
        // push I + A + B
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_I), Key::Raw(KEY_A), Key::Raw(KEY_B)]), vec![Key::Con(KEY_I), Key::Con(KEY_ENTER)]);

        let mut rule = Rules::new("".as_bytes()).unwrap().remove("").unwrap();
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
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_B), Key::Raw(KEY_ALT)]), vec![Key::Con(KEY_ENTER)]);
        // push C + ALT = 'SHIFT
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_C), Key::Raw(KEY_ALT)]), vec![Key::Con(KEY_SHIFT)]);
        // push B + C + ALT = 'ENTER + 'SHIFT
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_B), Key::Raw(KEY_C), Key::Raw(KEY_ALT)]), vec![Key::Con(KEY_ENTER), Key::Con(KEY_SHIFT)]);

        // 'ENTER + 'SHIFT -> 'A
        rule.list.push(KeyRule::new(vec![Key::Con(KEY_ENTER), Key::Con(KEY_SHIFT)], vec![Key::Con(KEY_A)]));
        // push B + C + ALT = 'A
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_B), Key::Raw(KEY_C), Key::Raw(KEY_ALT)]), vec![Key::Con(KEY_A)]);

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
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_B), Key::Raw(KEY_C), Key::Raw(KEY_ALT), Key::Raw(KEY_BACKSPACE), Key::Raw(KEY_TAB)]), vec![Key::Con(KEY_A), Key::Con(KEY_LEFT), Key::Con(KEY_RIGHT), Key::Con(KEY_UP), Key::Con(KEY_DOWN)]);

        // 'A + 'LEFT + 'RIGHT + 'UP + 'DOWN -> 'F
        rule.list.push(
            KeyRule::new(
                vec![Key::Con(KEY_A), Key::Con(KEY_LEFT), Key::Con(KEY_RIGHT), Key::Con(KEY_UP), Key::Con(KEY_DOWN)],
                vec![Key::Con(KEY_F)]
            )
        );
        // push B + C + ALT + BACKSPACE + TAB = 'F
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_B), Key::Raw(KEY_C), Key::Raw(KEY_ALT), Key::Raw(KEY_BACKSPACE), Key::Raw(KEY_TAB)]), vec![Key::Con(KEY_F)]);
    }
}