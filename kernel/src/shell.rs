//! 实现简单的shell

use crate::driver::fb::{clear, delete};
use crate::task::sleep::TIMER_TICK;
use crate::task::{executor, sleep, Task};
use crate::test::{exit_qemu, QemuExitCode};
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::str::FromStr;
use core::sync::atomic::Ordering;
use spin::{Lazy, Mutex};

/// 命令行缓冲
static CMD_BUF: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

/// 环境变量
static ENV: Lazy<Mutex<BTreeMap<String, String>>> = Lazy::new(|| Mutex::new(BTreeMap::new()));

/// 处理键盘输入
pub fn shell_input(chr: char) {
    let mut command = CMD_BUF.lock();

    match chr as u8 {
        10 => {
            // 处理换行
            fb_print!("\n");

            // 命令缓冲非空，执行命令
            let cmd = command.trim();
            if cmd.len() != 0 {
                let tokens: Vec<String> =
                    cmd.split_whitespace().map(|str| str.to_string()).collect();
                run_command(tokens);
            }
            command.clear();
            fb_print!("[rust_os] >>> ");
        }
        8 => {
            // 处理删除
            if command.len() != 0 {
                command.pop();
                // 退格、打印空格、再退格
                delete();
                fb_print!(" ");
                delete();
            }
        }
        other => {
            // 处理其它字符, 加入命令缓冲区并输出
            command.push(other as char);
            fb_print!("{}", other as char);
        }
    };
}

fn run_command(tokens: Vec<String>) {
    // 解析命令
    match tokens[0].as_str() {
        "clear" => {
            clear();
        }
        "echo" => {
            if tokens.len() < 2 {
                fb_println!("Error: expect a string to echo, example usage: echo hello");
                return;
            }
            fb_println!("{}", tokens[1]);
        }
        "env" => {
            let env = ENV.lock();
            for (key, value) in env.clone().into_iter() {
                fb_println!("{}={}", key, value);
            }
            if env.is_empty() {
                fb_println!("No environment variables");
            }
        }
        "exit" => {
            exit_qemu(QemuExitCode::Success);
        }
        "export" => {
            if tokens.len() < 2 {
                fb_println!("Error: expect an env setting, example usage: export MODE=async");
                return;
            }
            let mut env = ENV.lock();
            if let Some((key, value)) = tokens[1].split_once('=') {
                env.insert(key.to_string(), value.to_string());
            } else {
                env.insert(tokens[1].to_string(), "".to_string());
            };
        }
        "help" => {
            fb_println!("Try commands:");
            fb_println!("\thelp");
            fb_println!("\tclear");
            fb_println!("\techo hi");
            fb_println!("\texport ENV=value");
            fb_println!("\tenv");
            fb_println!("\tsleep 20 echo hi");
            fb_println!("\treboot");
            fb_println!("\texit (qemu only)");
            fb_println!("");
            fb_println!("Test sync/async modes:");
            fb_println!("\tsleep 100 echo hi (try to move mouse while waiting)");
            fb_println!("\texport MODE=async");
            fb_println!("\tsleep 100 echo hi (try to move mouse while waiting)");
            fb_println!("");
        }
        "reboot" => unsafe {
            core::arch::asm!(
                "cli
                mov al, 0xfe
                out 0x64, al"
            );
        },
        "sleep" => {
            if tokens.len() < 2 {
                fb_println!("Error: expect a sleep ticks, example usage: sleep 10 echo hello");
                return;
            }
            let ticks = match usize::from_str(tokens[1].as_str()) {
                Ok(ticks) => ticks,
                Err(_) => {
                    fb_println!("Error: can not convert to number: {}", tokens[1]);
                    return;
                }
            };
            if tokens.len() < 3 {
                fb_println!("Error: expect a target command after sleeping, example usage: sleep 10 echo hello");
                return;
            }

            let env = ENV.lock();
            if env.get("MODE").is_some_and(|value| value == "async") {
                // 异步运行命令
                let _ = executor::SPAWNED_TASKS
                    .lock()
                    .push(Task::new(sleep::sleep(ticks, move || {
                        run_command(tokens[2..].to_vec())
                    })));
            } else {
                // 同步运行命令
                TIMER_TICK.store(ticks, Ordering::Relaxed);
                loop {
                    if TIMER_TICK.load(Ordering::Relaxed) == 0 {
                        break;
                    }
                }
                run_command(tokens[2..].to_vec());
            }
        }
        "testasm" => {
            // let mut reg: i32;
            unsafe {
                // core::arch::asm!("mov rax, rax", out("rip") reg);
                // core::arch::asm!("mov rax, 0x60", "xor rdi, rdi", "syscall");
                core::arch::asm!("int 0x80");
            }
            // fb_println!("rax is {}", reg);
        }
        _ => {
            fb_println!("Error: unexpected command");
        }
    };
}
