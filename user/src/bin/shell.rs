#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::proc_create;
use user_lib::proc_wait;
use user_lib::{print::getchar, println};

const MAX_CMD_LEN: usize = 256;
const LF: u8 = b'\n';
const CR: u8 = b'\r';
const DL: u8 = b'\x7f';
const BS: u8 = b'\x08';

#[no_mangle]
pub fn main() -> i32 {
    println!("[Shell] Entered shell now!");
    let mut line_buffer = [0; MAX_CMD_LEN];
    let mut cursor = 0;
    print!("[Shell] >> ");
    loop {
        let ch = getchar();
        match ch {
            LF | CR => {
                println!("");
                if cursor > 0 {
                    line_buffer[cursor] = b'\0';
                    let path = core::str::from_utf8(&line_buffer[..cursor]).unwrap();
                    let (pid, _) = proc_create(path, path, None);
                    if pid == usize::MAX {
                        println!("[Shell] No such elf file!");
                    } else {
                        proc_wait(pid);
                    }
                }
                cursor = 0;
                print!("[Shell] >> ");
            }
            BS | DL => {
                // 退格打印空格再退格
                if cursor > 0 {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    cursor -= 1;
                }
            }
            _ => {
                print!("{}", ch as char);
                line_buffer[cursor] = ch;
                cursor += 1;
            }
        }
    }
}
