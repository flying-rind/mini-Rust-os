#![no_std]
#![no_main]

use user_lib::{exec, fork};

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    println!("Hello, world!");
    // let mut line: [u8; 256] = "02power".as_bytes().try_into().unwrap();
    let path = "02power";
    let pid = fork();
    if pid == 0 {
        if exec(path) == -1 {
            println!("FUCK the os");
            return -4;
        }
    }
    0x114 + 514
}
