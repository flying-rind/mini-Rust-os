#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

const LF: u8 = b'\n';
const CR: u8 = b'\r';
const DL: u8 = b'\x7f';
const BS: u8 = b'\x08';

use alloc::string::String;
use alloc::vec::Vec;
use user_lib::console::getchar;
use user_lib::{close, dup, exec, fork, open, pipe, waitpid, OpenFlags};

/// 命令行参数
#[derive(Debug)]
struct CmdArguments {
    input: String,
    output: String,
    args_copy: Vec<String>,
    args_addr: Vec<*const u8>,
}

impl CmdArguments {
    pub fn new(command: &str) -> Self {
        let args: Vec<&str> = command.split(' ').collect();
        let mut args_copy: Vec<String> = args
            .iter()
            .filter(|&args| !args.is_empty())
            .map(|&arg| {
                let mut string = String::new();
                string.push_str(arg);
                string.push('\0');
                string
            })
            .collect();

        // 重定向输入
        let mut input = String::new();
        if let Some((idx, _)) = args_copy
            .iter()
            .enumerate()
            .find(|(_, arg)| arg.as_str() == "<\0")
        {
            input = args_copy[idx + 1].clone();
            args_copy.drain(idx..=idx + 1);
        }

        // 重定向输出
        let mut output = String::new();
        if let Some((idx, _)) = args_copy
            .iter()
            .enumerate()
            .find(|(_, arg)| arg.as_str() == ">0")
        {
            output = args_copy[idx + 1].clone();
            args_copy.drain(idx..=idx + 1);
        }

        let mut args_addr: Vec<*const u8> = args_copy.iter().map(|arg| arg.as_ptr()).collect();
        args_addr.push(core::ptr::null::<u8>());

        Self {
            input,
            output,
            args_copy,
            args_addr,
        }
    }
}

#[no_mangle]
pub fn main() -> i32 {
    println!("Rust user shell");
    let mut line: String = String::new();
    print!(">> ");

    loop {
        let c = getchar();
        match c {
            LF | CR => {
                println!("");
                if !line.is_empty() {
                    // 按照|分割命令行参数，每个|表示前一个进程的输出连接到后一个进程输入
                    let splited: Vec<&str> = line.as_str().split('|').collect();
                    let process_arguments_list: Vec<CmdArguments> =
                        splited.iter().map(|&cmd| CmdArguments::new(cmd)).collect();
                    let mut valid = true;
                    // 检查管道是否能够建立
                    for (i, process_args) in process_arguments_list.iter().enumerate() {
                        // 第一个程序输出不能重定向
                        if i == 0 {
                            if !process_args.output.is_empty() {
                                valid = false;
                            }
                        // 最后一个程序输入不能重定向
                        } else if i == process_arguments_list.len() - 1 {
                            if !process_args.input.is_empty() {
                                valid = false;
                            }
                        }
                        // 中间的程序输入输出都不能重定向
                        else if !process_args.output.is_empty() || !process_args.input.is_empty()
                        {
                            valid = false;
                        }
                    }
                    // 只有一个程序时不需要建立管道
                    if process_arguments_list.len() == 1 {
                        valid = true;
                    }

                    // 管道不能建立
                    if !valid {
                        println!("Invalid command: Inputs/Outputs cannot be correctly binded");
                    } else {
                        // 创建管道
                        let mut pipes_fd: Vec<[usize; 2]> = Vec::new();
                        if !process_arguments_list.is_empty() {
                            for _ in 0..process_arguments_list.len() - 1 {
                                let mut pipe_fd = [0usize; 2];
                                pipe(&mut pipe_fd);
                                pipes_fd.push(pipe_fd);
                            }
                        }
                        let mut children: Vec<isize> = Vec::new();
                        // 对每个进程，使用fork和exec创建执行程序
                        for (i, process_argument) in process_arguments_list.iter().enumerate() {
                            let pid = fork();
                            // 子进程
                            if pid == 0 {
                                let input = &process_argument.input;
                                let output = &process_argument.output;
                                let args_copy = &process_argument.args_copy;
                                let args_addr = &process_argument.args_addr;

                                // 重定向输入
                                if !input.is_empty() {
                                    let input_fd = open(input, OpenFlags::RDONLY);
                                    if input_fd == -1 {
                                        println!("Error when opening file {}", input);
                                        return -4;
                                    }
                                    // 关闭标准输入
                                    close(0);
                                    // 使fd0也指向打开的inputfd
                                    assert_eq!(dup(input_fd as usize), 0);
                                    // 移除inputfd描述符
                                    close(input_fd as _);
                                }

                                // 重定向输出
                                if !output.is_empty() {
                                    let output_fd = open(output, OpenFlags::WRONLY);
                                    if output_fd == -1 {
                                        println!("Error when opening file {}", output);
                                        return -4;
                                    }
                                    // 关闭标准输出
                                    close(1);
                                    // 使fd1也指向打开的outputfd
                                    assert_eq!(dup(output_fd as _), 1);
                                    close(output_fd as _);
                                }

                                // 从上一个进程接受输入
                                if i > 0 {
                                    close(0);
                                    // 管道读端的fd
                                    let read_end = pipes_fd.get(i - 1).unwrap()[0];
                                    assert_eq!(dup(read_end), 0);
                                }
                                // 将输出发送到下一个进程
                                if i < process_arguments_list.len() - 1 {
                                    close(1);
                                    // 管道写端fd
                                    let write_end = pipes_fd.get(i).unwrap()[1];
                                    assert_eq!(dup(write_end), 1);
                                }
                                // 从文件表中移除从父进程(shell)继承的所有管道文件
                                for pipe_fd in pipes_fd.iter() {
                                    close(pipe_fd[0]);
                                    close(pipe_fd[1]);
                                }
                                // 执行应用程序
                                if exec(args_copy[0].as_str(), args_addr.as_slice()) == -1 {
                                    println!("Error when executing!");
                                    return -4;
                                }
                                unreachable!();
                            // 父进程
                            } else {
                                children.push(pid);
                            }
                        }
                        // shell程序自身关闭管道
                        for pipe_fd in pipes_fd.iter() {
                            close(pipe_fd[0]);
                            close(pipe_fd[1]);
                        }
                        let mut exit_code: i32 = 0;
                        // shell程序回收子进程
                        for pid in children.into_iter() {
                            let exit_pid = waitpid(pid as usize, &mut exit_code);
                            assert_eq!(pid, exit_pid);
                        }
                    }
                    // 清空已输入字符
                    line.clear();
                }
                print!(">> ");
            }

            BS | DL => {
                if !line.is_empty() {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    line.pop();
                }
            }

            _ => {
                print!("{}", c as char);
                line.push(c as char);
            }
        }
    }
}
