#![no_std]
#![no_main]

use user_lib::ls;

extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    ls();
    1
}
