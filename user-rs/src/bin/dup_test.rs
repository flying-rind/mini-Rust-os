#![no_std]
#![no_main]

extern crate user_lib;
use user_lib::dup;
use user_syscall::println;

#[no_mangle]
// fn main(_argc: usize, _argv: &[&str]) -> usize {
fn main() -> isize {
    dup(1);
    1
}
