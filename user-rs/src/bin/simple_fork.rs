#![no_std]
#![no_main]

use user_lib::{exec, exit, fork, wait4};
use user_syscall::println;

extern crate alloc;
extern crate user_lib;

#[no_mangle]
fn main() -> isize {
    let pid = fork().expect("Fork failed");
    if pid == 0 {
        println!("I am child");
        exit(1).expect("Failed to exit");
    } else {
        println!("I am parent");
        let mut exit_code: i32 = 0;
        wait4(pid, &mut exit_code).expect("wait4 failed");
    }
    1
}
