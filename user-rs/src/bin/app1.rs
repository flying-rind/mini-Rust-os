#![no_std]
#![no_main]

extern crate user_lib;
// use user_syscall::hybrid::test_cstr;
use user_syscall::println;

#[no_mangle]
// fn main(_argc: usize, _argv: &[&str]) -> usize {
fn main() -> isize {
    println!("[App1]: I am app1");
    0
}
