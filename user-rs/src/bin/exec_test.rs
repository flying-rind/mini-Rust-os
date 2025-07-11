#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use user_lib::*;

#[no_mangle]
fn main() -> isize {
    println!("[Exec_test]: I'm exec_test, trying to exec app1");
    let _ = exec("app1\0", &["arg1\0", "arg2\0"], &[]);
    1
}
