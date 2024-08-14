#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use user_lib::close;
use user_lib::exec;
use user_lib::make_pipe;
use user_lib::proc_wait;
use user_lib::{print::getchar, println};
use user_syscall::dup;
use user_syscall::fork;
use user_syscall::open;
use user_syscall::OpenFlags;

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
                // s.push('\0');
                s
            })
            .collect();
        // 重定向输入
        let mut input = String::new();
        if let Some((idx, _)) = args.iter().enumerate().find(|(_, arg)| arg.as_str() == "<") {
            input = args[idx + 1].clone();
            // 去掉< 和 输入文件
            args.drain(idx..=idx + 1);
        }
        // 重定向输出
        let mut output = String::new();
        if let Some((idx, _)) = args.iter().enumerate().find(|(_, arg)| arg.as_str() == ">") {
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
                println!("");
                if line.is_empty() {
                    print!("[Shell] >> ");
                    continue;
                }
                let splited: Vec<&str> = line.as_str().split('|').collect();
                let process_argmuments: Vec<ProcArguments> = splited
                    .iter()
                    .map(|&args| ProcArguments::new(args))
                    .collect();
                // 检查管道是否能够建立
                for (i, process_args) in process_argmuments.iter().enumerate() {
                    // 只有一个进程时不检查
                    if process_argmuments.len() == 1 {
                        break;
                    }
                    // 第一个程序不能重定向输出
                    if i == 0 {
                        if !process_args.output.is_empty() {
                            println!("Error: Cannot redirect input for first process");
                            continue;
                        } else if i == process_argmuments.len() - 1 {
                            if !process_args.input.is_empty() {
                                println!("Error: Cannot redirect output for last process");
                                continue;
                            }
                        } else {
                            if !process_args.output.is_empty() || !process_args.input.is_empty() {
                                println!("Error: Cannot redirect input/output for middle process");
                                continue;
                            }
                        }
                    }
                }
                // 建立管道
                let mut pipes_fd: Vec<(usize, usize)> = Vec::new();
                if !process_argmuments.is_empty() {
                    for _ in 0..process_argmuments.len() - 1 {
                        let pipe_fd = make_pipe();
                        pipes_fd.push(pipe_fd);
                    }
                }
                let mut children: Vec<usize> = Vec::new();
                // 创建进程
                for (i, process_arg) in process_argmuments.iter().enumerate() {
                    let pid = fork();
                    // 子进程
                    if pid == 0 {
                        let input = &process_arg.input;
                        let output = &process_arg.output;
                        let args = &process_arg.args;

                        // 重定向输入
                        if !input.is_empty() {
                            let input_fd = open(input, OpenFlags::RDONLY);
                            if input_fd == None {
                                println!("Error when opening file {}", input);
                                return -4;
                            }
                            // 关闭标准输入
                            close(0);
                            assert_eq!(dup(input_fd.unwrap()), Some(0));
                            // 标准输入改为input_fd
                            close(input_fd.unwrap());
                        }
                        // 重定向输出
                        if !output.is_empty() {
                            let output_fd = open(output, OpenFlags::WRONLY | OpenFlags::CREATE);
                            if output_fd == None {
                                println!("Error when opening file {}", output);
                                return -4;
                            }
                            // 关闭标准输出
                            close(1);
                            assert_eq!(dup(output_fd.unwrap()), Some(1));
                            // 标准输入改为input_fd
                            close(output_fd.unwrap());
                        }
                        // 从管道读端接受输入
                        if i > 0 {
                            close(0);
                            let read_end = (&pipes_fd[i - 1].0).clone();
                            assert_eq!(dup(read_end), Some(0));
                        }
                        // 输出输送到管道写端
                        if i < process_argmuments.len() - 1 {
                            close(1);
                            let write_end = pipes_fd[i].1;
                            assert_eq!(dup(write_end), Some(1));
                        }
                        // 从文件表中移除从父进程(shell)继承的所有管道文件
                        for pipe_fd in &pipes_fd {
                            close(pipe_fd.0);
                            close(pipe_fd.1);
                        }
                        // 执行应用程序
                        if exec(args[0].as_str(), Some(args)).0 == usize::MAX {
                            println!("Error when executing!");
                            return -4;
                        }
                        unreachable!();
                    // 父进程
                    } else {
                        children.push(pid);
                    }
                }
                // shell进程关闭所有管道
                for pipe_fd in &pipes_fd {
                    close(pipe_fd.0);
                    close(pipe_fd.1);
                }
                // 等待所有子进程结束
                for pid in &children {
                    proc_wait(*pid);
                }
                // 清空本行
                line.clear();
                print!("[Shell] >> ");
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
