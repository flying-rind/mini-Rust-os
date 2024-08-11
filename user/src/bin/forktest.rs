#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::fork;
use user_lib::proc_wait;

#[no_mangle]
fn main() -> usize {
    let pid = fork();
    if pid == 0 {
        println!("I am child process");
    } else {
        println!("I am parent process");
        proc_wait(pid);
    }
    println!("Fork test passed!");
    0
}