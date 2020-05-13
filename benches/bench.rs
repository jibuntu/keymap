#[macro_use]
extern crate lazy_static;
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

// key_converterモジュールのベンチマークテストをする
#[path = "../src/key_converter/mod.rs"]
mod key_converter;
use key_converter::KeyConverter;
pub fn key_converter_benchmark(c: &mut Criterion) {
    let mut kc = KeyConverter::new();
    let code = Keycode::new();
    let key_a = code.from_keyword("A").unwrap();
    let key_n = code.from_keyword("N").unwrap();
    let key_h = code.from_keyword("H").unwrap();
    let key_ctrl = code.from_keyword("LEFTCTRL").unwrap();
    
    // A -> 'Bの変換が行われる
    c.bench_function("KeyConverter 1", |b| b.iter(|| {
        kc.push(key_a);
        kc.leave(key_a);
    }));

    // N -> 'L, H -> 'I, LEFTCTRL + 'L -> 'LEFT, LEFTCTRL + 'I -> 'RIGHT
    c.bench_function("KeyConverter 2", |b| b.iter(|| {
        kc.push(key_n);
        kc.push(key_h);
        kc.push(key_ctrl);
        kc.leave(key_ctrl);
        kc.leave(key_h);
        kc.leave(key_n);
    }));
}

criterion_group!(benches, rules_benchmark, key_converter_benchmark);
criterion_main!(benches);