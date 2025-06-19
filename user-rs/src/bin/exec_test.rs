#![no_std]
#![no_main]

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use user_lib::exec;

extern crate alloc;
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    let _ = exec(
        "app1\0",
        vec!["app1\0".to_string(), "app2\0".to_string()],
        Vec::new(),
    );
    1
}
