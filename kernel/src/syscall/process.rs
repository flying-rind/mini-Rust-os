//! 与进程相关的系统调用实现
use super::*;
use crate::process;
use crate::syscall::uaccess::read_cstr;
use crate::syscall::EFAULT;
use crate::trap::SyscallFrame;

pub fn sys_exit(exit_code: i32) -> isize {
    process::current().exit(exit_code);
}

pub fn sys_yield() -> isize {
    process::current_yield();
    0
}

pub fn sys_getpid() -> isize {
    process::current().proc.pid as _
}

pub fn sys_fork(_f: &SyscallFrame) -> isize {
    process::current().proc.fork().pid as _
}

pub fn sys_exec(path: *const u8, args: *const *const u8) -> isize {
    let path = if let Some(x) = read_cstr(path) {
        x
    } else {
        return EFAULT;
    };
    let args = if let Some(x) = read_cstr_array(args) {
        x
    } else {
        return EFAULT;
    };
    process::current().proc.exec(&path, args)
}

/// If there is no child process has the same pid as the given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_p: *mut u32) -> isize {
    let (pid, exit_code) = process::current().proc.waitpid(pid);
    if pid >= 0 && !exit_code_p.is_null() {
        exit_code_p.write_user(exit_code as _);
    }
    pid
}
