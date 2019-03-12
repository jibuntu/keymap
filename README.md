# What is "keymap"?
This program is a command line tool.

# What can we do using this?
You can change keyboard layout.

# usage
You must built this program before using.
```bash
cargo build
```
<!-- ファイルを指定して実行する -->
```
sudo ./target/debug/keymap file_path
```

# Example of keymap file
* You can see all keys in keymap/src/rules/keycode/mod.rs.
```
# This line is comment.

# Change "A" key to "B" key
A -> B

# Change leftAlt to leftCtrl
LeftAlt -> LeftCtrl

LeftCtrl + I -> Up
LeftCtrl + K -> Down
LeftCtrl + E -> Esc
Tab + S -> LeftCtrl + S
```
