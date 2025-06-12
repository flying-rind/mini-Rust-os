#![no_std]
#![no_main]

use user_syscall::println;

extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    println!("Hello world!, i'm app1");
    1
}
