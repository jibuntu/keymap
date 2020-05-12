extern crate criterion;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashSet;
use std::iter::FromIterator;


// rulesモジュールのベンチマークテストをする
// Rules::filter関数はキーが押されるたびに呼ばれるので速度は速いほうがいい
#[path = "../src/key_converter/rules/mod.rs"]
mod rules;
use rules::Rules;
use rules::KeyRule;
use rules::Key;
use rules::keycode::Keycode;

pub fn rules_benchmark(c: &mut Criterion) {
    let code = Keycode::new();
    let key_a = code.from_keyword("A").unwrap();
    let key_b = code.from_keyword("B").unwrap();
    let key_c = code.from_keyword("C").unwrap();
    let key_d = code.from_keyword("D").unwrap();
    let key_e = code.from_keyword("E").unwrap();
    let key_enter = code.from_keyword("ENTER").unwrap();
    let key_ctrl = code.from_keyword("LEFTCTRL").unwrap();
    let key_alt = code.from_keyword("LEFTALT").unwrap();

    let v = vec![
        // A -> 'C
        KeyRule::new(vec![Key::Raw(key_a)], vec![Key::Con(key_c)]),
        // B -> 'D
        KeyRule::new(vec![Key::Raw(key_b)], vec![Key::Con(key_d)]),
        // CTRL + 'C -> 'ENTER
        KeyRule::new(vec![Key::Raw(key_ctrl), Key::Con(key_c)], vec![Key::Con(key_enter)]),
        // 'ENTER + 'D -> 'ALT
        KeyRule::new(vec![Key::Con(key_enter), Key::Con(key_d)], vec![Key::Con(key_alt)]),
    ];
    let rules = Rules::from_vec(v);
    let mut k = HashSet::new();
    k.insert(Key::Raw(key_a));

    // A -> 'Cの変換が行われる
    c.bench_function("Rule::filter 1", |b| b.iter(|| rules.filter(&k)));

    // A -> 'C, CTRL + 'C -> 'ENTER
    k.insert(Key::Raw(key_ctrl));
    c.bench_function("Rule::filter 2", |b| b.iter(|| rules.filter(&k)));

    // 最終的には'altに変換される
    k.insert(Key::Raw(key_b));
    assert_eq!(rules.filter(&k), HashSet::from_iter(vec![Key::Con(key_alt)].into_iter()));
    c.bench_function("Rule::filter 3", |b| b.iter(|| rules.filter(&k)));
}

criterion_group!(benches, rules_benchmark);
criterion_main!(benches);