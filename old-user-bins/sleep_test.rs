#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::{get_time, println, sleep};

#[no_mangle]
pub fn main() -> i32 {
    println!("into sleep test");
    let start = get_time();
    println!("current time_ms = {}", start);
    sleep(100);
    let end = get_time();
    println!(
        "time_ms = {} after sleeping 100 ticks, delta = {}ms!",
        end,
        end - start
    );
    println!("sleep_test passed");
    0
}
