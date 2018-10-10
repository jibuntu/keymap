extern crate gcc;

fn main(){
    gcc::Config::new()
        .file("src/c/ioctl_eviocgrab.c")
        .include("src")
        .compile("libioctl_eviocgrab");
    gcc::Config::new()
        .file("src/c/virtual_keyboard.c")
        .include("src")
        .compile("virtual_keyboard");
}