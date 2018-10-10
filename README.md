# keymap
This is a tool to change key code on linux.

# usage
```bash
cargo build
./target/debug/keymap file_path
```
file_path must be an absolute oath.

# Example of keymap file
* You can see all keys in keymap/src/rules/keycode/mod.rs.
```
# This line is comment.

# Change "A" key to "B" key
A -> B

# Change leftAlt to leftCtrl
leftAlt -> leftCtrl

# Up is inputed when left control key and "K" key are pushed.
leftCtrl + K -> up
```
