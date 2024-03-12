//! 与进程相关的系统调用实现
use crate::process::*;

pub fn sys_exit(exit_code: i32) -> isize {
    current_exit(exit_code);
}

pub fn sys_yield() -> isize {
    current_yield();
    0
}
