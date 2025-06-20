#![no_std]
#![no_main]

extern crate user_lib;
// use user_syscall::hybrid::test_cstr;
use user_syscall::monolithic::test_cstr;

#[no_mangle]
// fn main(_argc: usize, _argv: &[&str]) -> usize {
fn main() -> isize {
    let _ = test_cstr(&["app1\0", "app2\0"]);
    0
}
