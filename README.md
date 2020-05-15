# Example
```bash
$ cat test/keymap.txt 
# you can change keyboard layout.

CapsLock -> 'LeftCtrl
LeftCtrl -> 'CapsLock

F -> 'A
J -> 'B


# You can use multiple keys when changing.

'LeftCtrl + 'A -> 'BackSpace

'LeftCtrl + 'B -> 'LeftCtrl + 'RightShift + 'T


# You can change rules when pushed key.

Q -> @RULE_1

@RULE_1
  F -> 'B
  J -> 'A

  LeftCtrl + Q -> @RULE_2

@RULE_2
  F -> 'C
  J -> 'D

  LeftCtrl + Q -> @RULE_1

$ cargo build --release
$ sudo target/release/keymap test/keymap.txt
```
You can see all keys in [keymap/src/key_converter/rules/keycode/mode.rs](https://github.com/jibuntu/keymap/blob/master/src/key_converter/rules/keycode/mod.rs)
