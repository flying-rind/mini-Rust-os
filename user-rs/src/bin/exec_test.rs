#![no_std]
#![no_main]

use alloc::vec::Vec;
use user_syscall::monolithic::exec;
use user_syscall::println;

extern crate alloc;
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    let _ = exec("123456\0", &["arg1\0", "arg2\0"], &["env1\0"]);
    1
}

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
