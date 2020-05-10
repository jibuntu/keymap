// extern crate gcc;
extern crate cc;

fn main(){
    cc::Build::new()
        .file("src/c/ioctl_eviocgrab.c")
        .include("src")
        .compile("libioctl_eviocgrab");
    cc::Build::new()
        .file("src/c/virtual_keyboard.c")
        .include("src")
        .compile("virtual_keyboard");
}