//! 任务管理类系统调用
use crate::monolithic::error::SysResult;
// use crate::monolithic::print::print;
use alloc::vec::Vec;
use core::ptr;

use super::*;

/// Copy current process, return child's PID.
pub fn fork() -> SysResult {
    sys_fork()
}

/// Replaces the current ** process ** with a new process image
///
/// `argv` is an array of argument strings passed to the new program.
/// `envp` is an array of strings, conventionally of the form `key=value`,
/// which are passed as environment to the new program.
///
/// NOTICE: `argv` & `envp` can not be NULL (different from Linux)
///
/// NOTICE: for multi-thread programs
/// A call to any exec function from a process with more than one thread
/// shall result in all threads being terminated and the new executable image
/// being loaded and executed.
///
/// FIXME: Ignore env for now.
pub fn exec(path: &str, argv: &[&str], _env: &[&str]) -> SysResult {
    let pathp = path.as_ptr() as *const u8;
    // This vec cannot be dropped because kernel will use it. :)
    let mut argv_ptrs: Vec<*const u8> = argv.iter().map(|&s| s.as_ptr()).collect();
    if argv.is_empty() {
        // no arg
        sys_exec(pathp, ptr::null(), ptr::null())
    } else {
        // Indicate the end of argv(*const u8)
        argv_ptrs.push(ptr::null());
        sys_exec(pathp, argv_ptrs.as_ptr(), ptr::null())
    }
}

/// Exit the current thread
pub fn exit(exit_code: usize) -> SysResult {
    sys_exit(exit_code)
}

/// Wait the process to exit.
/// Return the PID. Store exit coe to `code` if it's not null.
pub fn wait4(pid: usize, wstatus: *mut i32) -> SysResult {
    sys_wait(pid, wstatus)
}
