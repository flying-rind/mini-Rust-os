#![no_std]
#![no_main]

use user_lib::{exec, fork, waitpid};

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    println!("Hello, world!");
    let pid = fork();
    if pid == 0 {
        if exec("02power") == -1 {
            println!("command not found");
            return -4;
        }
    } else {
        let mut exit_code: i32 = 0;
        let exit_pid: isize = waitpid(pid as usize, &mut exit_code);
        assert_eq!(pid, exit_pid);
        println!("Process {} exited with code {}", pid, exit_code);
    }
    0x114 + 514
}
