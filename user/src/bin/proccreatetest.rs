#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use crate::alloc::string::ToString;
use alloc::vec;
use user_lib::proc_create;

#[no_mangle]
fn main(argc: usize, argv: &[&str]) -> usize {
    let args = vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()];
    proc_create("printargs", "printargs", Some(args));
    0
}
