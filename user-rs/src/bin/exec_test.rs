#![no_std]
#![no_main]

use alloc::vec::Vec;
use user_syscall::monolithic::exec;
use user_syscall::println;

extern crate alloc;
extern crate user_lib;

#[no_mangle]
fn main() -> isize {
    println!("[Exec_test]: I'm exec_test, trying to exec app1");
    let _ = exec("app1\0", &["arg1\0", "arg2\0"], &["env1\0"]);
    1
}

#[allow(unused)]
fn test(argv: &[&str]) {
    let ptrs: Vec<*const u8> = argv.iter().map(|&s| s.as_ptr()).collect();
    let argvp = ptrs.as_ptr();
    unsafe {
        println!(
            "argvp = {:?}, *argvp = {:?}, **argvp = {}",
            argvp, *argvp, **argvp as char
        );
    }
    println!("test passed")
}
