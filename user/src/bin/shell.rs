#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use user_lib::proc_create;
use user_lib::proc_wait;
use user_lib::{print::getchar, println};

use alloc::string::String;
use alloc::vec::Vec;

const LF: u8 = b'\n';
const CR: u8 = b'\r';
const DL: u8 = b'\x7f';
const BS: u8 = b'\x08';

/// 一个进程的参数
struct ProcArguments {
    /// 输入文件
    input: String,
    /// 输出文件
    output: String,
    /// 程序参数
    args: Vec<String>,
}

impl ProcArguments {
    /// 给定一个进程的参数，解析其参数和输入输出
    pub fn new(command: &str) -> Self {
        // 收集一个进程的参数
        let args: Vec<&str> = command.split(' ').collect();
        let mut args: Vec<String> = args
            .iter()
            .filter(|&arg| !arg.is_empty())
            .map(|&arg| {
                let mut s = String::new();
                s.push_str(arg);
                s.push('\0');
                s
            })
            .collect();
        // 重定向输入
        let mut input = String::new();
        if let Some((idx, _)) = args
            .iter()
            .enumerate()
            .find(|(_, arg)| arg.as_str() == "<\0")
        {
            input = args[idx + 1].clone();
            // 去掉< 和 输入文件
            args.drain(idx..=idx + 1);
        }
        // 重定向输出
        let mut output = String::new();
        if let Some((idx, _)) = args
            .iter()
            .enumerate()
            .find(|(_, arg)| arg.as_str() == ">0")
        {
            output = args[idx + 1].clone();
            args.drain(idx..=idx + 1);
        }
        Self {
            input,
            output,
            args,
        }
    }
}

#[no_mangle]
pub fn main() -> i32 {
    println!("[Shell] Entered shell now!");
    let mut line = String::new();
    print!("[Shell] >> ");
    loop {
        let ch = getchar();
        match ch {
            LF | CR => {
                unimplemented!();
            }
            BS | DL => {
                // 退格打印空格再退格
                if !line.is_empty() {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    line.pop();
                }
            }
            _ => {
                print!("{}", ch as char);
                line.push(ch as char);
            }
        }
    }
}
