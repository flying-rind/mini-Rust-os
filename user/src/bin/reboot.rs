#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_syscall::close;
use user_syscall::read;
use user_syscall::write;
use user_syscall::{open, OpenFlags};

static WRITE_TIMES: usize = 20;

#[no_mangle]
fn main() -> i32 {
    let filea = "filea";
    let fd = open(filea, OpenFlags::CREATE | OpenFlags::WRONLY);
    assert!(fd.is_some());
    let fd = fd.unwrap();
    let test_str = "a";

    for _ in 0..WRITE_TIMES {
        write(fd, test_str.as_bytes());
    }

    let fd = open(filea, OpenFlags::RDONLY);
    assert!(fd.is_some());
    let fd = fd.unwrap();
    let mut buffer = [0u8; 100];
    let read_len = read(fd, &mut buffer).unwrap();
    close(fd);
    println!(
        "Write {} bytes, read {} bytes, {} requestes dropped totally",
        WRITE_TIMES,
        read_len,
        WRITE_TIMES - read_len
    );
    println!("reboot test passed!");
    0
}
