//! 与进程相关的系统调用实现
use super::*;
use crate::trap::SyscallFrame;

pub fn sys_exit(exit_code: i32) -> isize {
    process::current().exit(exit_code);
}

pub fn sys_yield() -> isize {
    process::current_yield();
    0
}

pub fn sys_getpid() -> isize {
    process::current().id as _
}

pub fn sys_fork(f: &SyscallFrame) -> isize {
    process::current().fork(f)
}

pub fn sys_exec(path: *const u8, f: &mut SyscallFrame) -> isize {
    let path = if let Some(x) = read_cstr(path) {
        x
    } else {
        return EFAULT;
    };
    process::current().exec(&path, f)
}

pub fn sys_waitpid(pid: isize, exit_code_p: *mut u32) -> isize {
    let (pid, exit_code) = process::current().waitpid(pid);
    if pid >= 0 && !exit_code_p.is_null() {
        exit_code_p.write_user(exit_code as _);
    }
    pid
}
