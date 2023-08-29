#![no_std]
#![no_main]

use user_lib::console::getchar;

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    let a = getchar();
    println!("Hello im program 6. bye, {}.", a);
    6
}
