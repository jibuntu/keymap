use std::io::Read;
use std::fs::File;
use std::io;
use regex::Regex;
use std::collections::{HashSet,HashMap};

mod keycode;
use self::keycode::Keycode;


fn get_left_keywords(left_match: &str) -> Option<HashSet<String>> {
    lazy_static! { static ref re: Regex = Regex::new(r"[^ +]+").unwrap(); }
    let keywords: HashSet<String> = re.find_iter(left_match).map(|keyword| keyword.as_str().to_string()).collect();

    if keywords.len() == 0 {
        return None;
    }

    Some(keywords)
}

fn get_right_keyword(right_match: &str) -> Option<String> {
    lazy_static! { static ref re: Regex = Regex::new(r"[^ ]+").unwrap(); }
    let mut keyword_iter = re.find_iter(right_match);
    
    let keyword = match keyword_iter.next() {
        Some(keyword) => keyword.as_str().to_string(),
        None => return None
    };

    // 必要以上にキーワードがあったらエラー
    if let Some(_) = keyword_iter.next() {
        return None;
    }

    Some(keyword)
}

pub struct ModifierRule {
    pub keys: HashSet<u16>,
    pub value: u16,
}

enum ValueConvertedLine {
    ModifierRule(ModifierRule),
    Keycode((u16, u16)),
}

fn convert_line(rule_str: &str) -> Option<ValueConvertedLine> {
    let keycode = Keycode::new();
    lazy_static! { static ref re: Regex = Regex::new("(.*)->(.*)").unwrap(); }
    let capture = match re.captures(rule_str) { 
        Some(capture) => capture,
        None => return None
    };

    let left_keycodes: HashSet<u16> = match get_left_keywords(&capture[1]) {
        Some(keywords) => {
            let mut left_keycodes: HashSet<u16> = HashSet::with_capacity(keywords.len());
            for keyword in &keywords {
                // 同じキーがあるときはだめ
                if left_keycodes.insert(keycode.from_keyword(keyword)?) == false {
                    return None
                }
            }
            left_keycodes
        },
        None => return None
    };

    let right_keycode = match get_right_keyword(&capture[2]) {
        Some(keyword) => keycode.from_keyword(&keyword)?,
        None => return None
    };

    if left_keycodes.len() == 1 {
        return Some(ValueConvertedLine::Keycode((*left_keycodes.iter().nth(0).unwrap(), right_keycode)));
    }

    return Some(ValueConvertedLine::ModifierRule(ModifierRule {
        keys: left_keycodes,
        value: right_keycode
    }))
}

fn line_is_empty_or_comment(line_: &str) -> bool {
    lazy_static! { static ref re: Regex = Regex::new("^ *$|^ *#").unwrap(); }
    re.is_match(line_)
}

pub struct Rules {
    modifier_rules: Vec<ModifierRule>,
    keycode_rules: HashMap<u16,u16>
}

impl Rules {
    pub fn from_file(path: &str) -> Option<Rules> {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(e) => { println!("Can't open: {:?}", e.kind()); return None; }
        };
        let mut is_err = 0;
        let mut data = String::new();
        let mut modifier_rules: Vec<ModifierRule> = Vec::new();
        let mut keycode_rules: HashMap<u16, u16> = HashMap::new();
        file.read_to_string(&mut data);

        for (i, data_line) in data.lines().enumerate() {
            if line_is_empty_or_comment(data_line) {
                continue;
            }

            if let Some(converted_data) = convert_line(data_line) {
                match converted_data {
                    ValueConvertedLine::ModifierRule(rule) => {
                        modifier_rules.push(rule);
                    },
                    ValueConvertedLine::Keycode((left, right)) => {
                        keycode_rules.insert(left, right);
                    }
                }
            }else{
                if is_err == 0 {
                    is_err = 1;
                    println!("Below lines could not be convert.");
                }
                println!(" {:3} | {}", i, data_line);
            }
        }

        Some(Rules{
            modifier_rules: modifier_rules,
            keycode_rules: keycode_rules
        })
    }

    pub fn change_keycode(&self, keycode: u16) -> Option<u16> {
        match self.keycode_rules.get(&keycode) {
            Some(keycode) => Some(*keycode),
            None => None
        }
    }

    pub fn contains_and_trigger(&self, pressed_keys: &HashSet<u16>, code: u16) -> Option<&ModifierRule> {
        for mod_rule in &self.modifier_rules {
            if mod_rule.keys.is_subset(&pressed_keys) && mod_rule.keys.contains(&code) {
                return Some(mod_rule);
            }
        }
        None
    }
}

#[test]
fn test_rules() {
    let rules = match Rules::from_file("/home/jibuntu/programming_language/rust/project/keymap/src/rules/test.keymap") {
        Some(rules) => rules,
        None => return
    };
}
