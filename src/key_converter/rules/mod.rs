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


/// ルールの構造体
#[derive(Debug, Clone, PartialEq)]
pub struct Rules {
    name: String,
    extend: Option<String>, // 継承するルール名
    list: Vec<KeyRule>,
}

impl Rules {
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

/// 文字列からルールを作成する
pub struct RulesParser {}

/// 読み出したルールを一時的に保持する構造体
pub struct ParsedRules {
    name: String,
    extend: Option<String>,
    line: usize,
    rule_list: Vec<(usize, KeyRule)>,
}

impl RulesParser {
    pub fn parse<R: Read>(mut r: R) -> Result<HashMap<String, Rules>, String> {
        let mut s = String::new();
        let mut parsed_rules_list = HashMap::new();
        let mut parsed_rules = ParsedRules {
            name: "".to_string(),
            extend: None,
            line: 0,
            rule_list: Vec::new(),
        };

        let _ = r.read_to_string(&mut s);

        // コメントを削除して、不要な行を削除する
        let lines = s.lines()
            .map(|l| l.split('#').next().unwrap())
            .map(|l| l.trim())
            .enumerate()
            .filter(|(_, l)| l.len() != 0);

        for (i, l) in lines {
            match l.chars().next().unwrap() {
                // '@'が来たらparsed_rulesをparsed_rules_listに追加して、
                // 新しくparsed_rulesを作成する
                '@' => match l.get(1..) {
                    Some(n) => {
                        parsed_rules_list.insert(parsed_rules.name.clone(),
                                                 parsed_rules);

                        // nameを取得する
                        let name = 
                            n.split(':').next().unwrap().trim().to_string();

                        // extendがあれば取得する
                        let extend = if let Some(ex) = n.split(':').nth(1) {
                            // "@"のみのときのルール名は空文字列
                            Some(ex.trim().get(1..).unwrap_or("").to_string())
                        } else {
                            None
                        };

                        // parsed_rulesを新しくする
                        parsed_rules = ParsedRules {
                            name,
                            extend,
                            line: i,
                            rule_list: Vec::new(),
                        };
                    },
                    None => return Err(format!("ルール名がありません: line {}", i))
                },
                _ => match KeyRule::from_str(l) {
                    Ok(k) => {
                        parsed_rules.rule_list.push((i, k));
                    },
                    Err(e) => return Err(format!("{}: line {}", e, i+1))
                }
            }
        }

        parsed_rules_list.insert(parsed_rules.name.clone(), parsed_rules);

        let mut rules_list = HashMap::new();
        for key in parsed_rules_list.keys() {
            // 重複などのときはエラーを出し、上書きなどをしながら
            // ルールのリストを作る
            let mut list: Vec<KeyRule> = Vec::new();
            let rules = match RulesParser::get_rule_rec(key, vec![], &parsed_rules_list) {
                Ok(r) => r,
                Err(e) => return Err(format!("'@{}' {}", key, e))
            };

            'outer: for (i, r) in rules {
                for rule in &mut list {
                    if !rule.compare_k(&r.k) {
                        continue
                    }

                    // オーバーライドがtrueだったら、ルールの値を書き換える
                    if r.ove {
                        *rule = r.clone();
                        continue 'outer
                    }

                    // そうでなければエラーを返す
                    return Err(format!("同じキーでルールを登録することはできません: line {}", i))
                }

                list.push(r);
            }

            // Rulesを追加する
            rules_list.insert(key.to_string(), Rules {
                name: key.to_string(),
                extend: parsed_rules_list.get(key).unwrap().extend.clone(),
                list
            });
        }

        Ok(rules_list)
    }

    /// 再帰的にルールを取得する
    fn get_rule_rec<'a>(name: &'a str, 
                    mut name_history: Vec<&'a str>,
                    parsed_rules_list: &'a HashMap<String, ParsedRules>) 
                    -> Result<Vec<(usize, KeyRule)>, String>
    {
        // すでに同じ名前があったらエラーを返す
        if name_history.contains(&name) {
            return Err("継承が循環しています".to_string())
        }

        name_history.push(&name);
        let mut rules = Vec::new();

        let parsed_rules = match parsed_rules_list.get(name) {
            Some(pr) => pr,
            None => return Ok(rules)
        };

        // 継承先があればそれを先に追加する
        if let Some(e) = &parsed_rules.extend {
            let mut r = RulesParser::get_rule_rec(
                &e, name_history, parsed_rules_list)?;
            rules.append(&mut r);
        }

        // 継承先のルールを追加したあとに自身を追加する
        rules.append(&mut parsed_rules.rule_list.clone());

        Ok(rules)
    }
}


#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::collections::HashMap;
    use super::Key;
    use super::KeyRule;
    use super::Rules;
    use super::key_rule::keycode::Keycode;
    use super::RulesParser;

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
    fn test_rules_parser() {
        let code = Keycode::new();

        let r = RulesParser::parse("".as_bytes()).unwrap().remove("").unwrap();
        assert_eq!(r.list, vec![]);

        if let Ok(_) = RulesParser::parse("->".as_bytes()) { panic!() }
        if let Ok(_) = RulesParser::parse("a->".as_bytes()) { panic!() } 
        if let Ok(_) = RulesParser::parse("->mm".as_bytes()) { panic!() } 

        let r = RulesParser::parse("A -> B".as_bytes()).unwrap().remove("").unwrap();
        assert_eq!(r.list, vec![
            KeyRule::new(vec![Key::Raw(code.from_keyword("A").unwrap())], vec![Key::Raw(code.from_keyword("B").unwrap())])
        ]);

        let mut r = RulesParser::parse(r#"
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

        let r = RulesParser::parse(r#"
        A -> 'B
        B -> 'A
        "#.as_bytes()).unwrap();
        let mut rlist = HashMap::new();
        rlist.insert("".to_string(), Rules { name: String::new(), extend: None, list: vec![
            KeyRule::new(vec![Key::Raw(code.from_keyword("A").unwrap())], vec![Key::Con(code.from_keyword("B").unwrap())]),
            KeyRule::new(vec![Key::Raw(code.from_keyword("B").unwrap())], vec![Key::Con(code.from_keyword("A").unwrap())]),
        ]});
        assert_eq!(r, rlist);

        let r = r#"
        A -> 'B
        @test
          A -> 'A
          A -> 'C
        "#.as_bytes();
        assert_eq!(RulesParser::parse(r), Err("同じキーでルールを登録することはできません: line 4".to_string()));


        let r = RulesParser::parse(r#"
        A -> 'B
        @test : @
          A -!> 'A
          A -!> 'C
        "#.as_bytes()).unwrap();
        let mut rlist = HashMap::new();
        rlist.insert("".to_string(), Rules { name: String::new(), extend: None, list: vec![
            KeyRule::new(vec![Key::Raw(code.from_keyword("A").unwrap())], vec![Key::Con(code.from_keyword("B").unwrap())]),
        ]});
        rlist.insert("test".to_string(), Rules { name: "test".to_string(), extend: Some("".to_string()), list: vec![
            KeyRule::with_ove(vec![Key::Raw(code.from_keyword("A").unwrap())], vec![Key::Con(code.from_keyword("C").unwrap())], true),
        ]});
        assert_eq!(r, rlist);

        let r = RulesParser::parse(r#"
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
        let r = RulesParser::parse(r#"
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

        let r = RulesParser::parse(r#"
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
            KeyRule::new(vec![Key::Raw(code.from_keyword("A").unwrap())], vec![Key::Con(code.from_keyword("B").unwrap())]),
            KeyRule::new(vec![Key::Raw(code.from_keyword("B").unwrap())], vec![Key::Con(code.from_keyword("A").unwrap())]),
            KeyRule::new(vec![Key::Raw(code.from_keyword("M").unwrap())], vec![Key::Con(code.from_keyword("N").unwrap())]),
            KeyRule::new(vec![Key::Raw(code.from_keyword("N").unwrap())], vec![Key::Con(code.from_keyword("M").unwrap())]),
        ]});
        assert_eq!(r, rlist);

        let r = r#"
        @RULE1 : @RULE2
        @RULE2 : @RULE3
        @RULE3 : @RULE1
        "#.as_bytes();
        // RULE1, RULE2, RULE3のいずれか
        match RulesParser::parse(r) {
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

        let rule = RulesParser::parse("".as_bytes()).unwrap().remove("").unwrap();
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_A)]), vec![Key::Raw(KEY_A)]);


        let mut rule = RulesParser::parse("".as_bytes()).unwrap().remove("").unwrap();
        // A -> 'H
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_A)], vec![Key::Con(KEY_H)]));
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_A)]), vec![Key::Con(KEY_H)]);

        let mut rule = RulesParser::parse("".as_bytes()).unwrap().remove("").unwrap();
        // A -> 'H
        rule.list.push(KeyRule::new(vec![Key::Raw(KEY_A)], vec![Key::Con(KEY_H)]));
        // 'H -> 'A
        rule.list.push(KeyRule::new(vec![Key::Con(KEY_H)], vec![Key::Con(KEY_A)]));
        assert_eq!(rule.filter(&hash![Key::Raw(KEY_A)]), vec![Key::Con(KEY_A)]);

        let mut rule = RulesParser::parse("".as_bytes()).unwrap().remove("").unwrap();
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

        let mut rule = RulesParser::parse("".as_bytes()).unwrap().remove("").unwrap();
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