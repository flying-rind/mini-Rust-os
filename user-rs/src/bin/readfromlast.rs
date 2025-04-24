#![no_std]
#![no_main]

use user_lib::read;

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    let mut buffer = [0u8; 30];
    let read_size = read(0, &mut buffer);
    assert!(read_size.is_some());
    let size = read_size.unwrap();
    print!("{}", core::str::from_utf8(&buffer[..size]).unwrap());
    1
}
