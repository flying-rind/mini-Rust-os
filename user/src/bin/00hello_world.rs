#![no_std]
#![no_main]

use user_lib::{exec, fork};

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    println!("Hello, world!");
    let a = fork();
    if a == 0 {
        exec("02power");
    }
    0x114 + 514
}
