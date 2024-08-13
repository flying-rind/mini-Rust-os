//! 任务管理类系统调用

use crate::sys_proc_exit;
use crate::*;
use alloc::{string::String, vec::Vec};

/// 退出当前进程
pub fn proc_exit(exit_code: usize) -> () {
    sys_proc_exit(exit_code);
}

/// 创建新进程
///
/// 若创建失败，则第一个返回值为usize::MAX
pub fn proc_create(name: &str, path: &str, args: Option<Vec<String>>) -> (usize, usize) {
    let name_ptr: usize = &name as *const &str as usize;
    let path_ptr = &path as *const &str as usize;
    let mut args_ptr = 0;
    if let Some(args) = &args {
        args_ptr = args as *const Vec<String> as usize;
    }
    sys_proc_create(name_ptr, path_ptr, args_ptr)
}

/// 替换当前进程的elf
///
/// 若创建失败，则第一个返回值为usize::MAX
pub fn exec(path: &str, args: Option<&Vec<String>>) -> (usize, usize) {
    // println!("In user exec path: {}", path);
    let path_ptr = &path as *const &str as usize;
    let mut args_ptr = 0;
    if let Some(args) = args {
        args_ptr = args as *const Vec<String> as usize;
    }
    sys_exec(path_ptr, args_ptr)
}

/// 当前线程等待一个进程结束
pub fn proc_wait(pid: usize) {
    let (ret0, _ret1) = sys_proc_wait(pid);
    if ret0 == 255 {
        // debug_write("SyscallError: waited process does not exist");
    }
}

/// 当前线程主动放弃CPU
pub fn current_yield() {
    sys_yield();
}

/// 创建线程，返回新线程tid，若失败返回None
pub fn thread_create(entry: usize, arg1: usize, arg2: usize) -> Option<usize> {
    let (tid, _ret1) = sys_thread_create(entry, arg1, arg2);
    if tid == usize::MAX {
        None
    } else {
        Some(tid)
    }
}

/// 当前线程退出
pub fn thread_exit() {
    sys_thread_exit();
}

/// 当前线程等待另一个线程
///
/// 只允许主线程使用，若非主线程则返回None
pub fn thread_join(tid: usize) -> Option<usize> {
    let (ret1, _) = sys_thread_join(tid);
    if ret1 == usize::MAX {
        return None;
    }
    Some(ret1)
}

/// 获取当前进程pid
pub fn get_pid() -> usize {
    let (pid, _) = sys_get_pid();
    pid
}

/// 获取当前线程tid
pub fn get_tid() -> usize {
    let (tid, _) = sys_get_tid();
    tid
}

/// 复制当前进程
pub fn fork() -> usize {
    let (pid, _) = sys_fork();
    pid
}
