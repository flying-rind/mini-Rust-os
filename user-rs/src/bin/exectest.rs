#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;
use alloc::string::ToString;
use alloc::vec;
use user_lib::exec;
use user_lib::fork;
use user_lib::proc_wait;

#[no_mangle]
fn main() -> usize {
    let pid = fork();
    if pid == 0 {
        let args = vec!["arg1".to_string(), "arg2".to_string()];
        exec("printargs", Some(&args));
    } else {
        println!("I am parent process");
        proc_wait(pid);
    }
    println!("Exec test passed!");
    0
}
