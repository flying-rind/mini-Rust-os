//! 任务管理类系统调用
use crate::monolithic::error::SysResult;
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
pub fn exec(path: &str, argv: &[&str], env: &[&str]) -> SysResult {
    let path = path.as_ptr() as *const u8;
    let argvp: *const *const u8 = if argv.is_empty() {
        ptr::null()
    } else {
        let ptrs: Vec<*const u8> = argv.iter().map(|&s| s.as_ptr()).collect();
        ptrs.as_ptr()
    };
    let envp = if env.is_empty() {
        ptr::null()
    } else {
        let mut ptrs: Vec<*const u8> = env.iter().map(|s| s.as_ptr()).collect();
        ptrs.push(ptr::null());
        let envp = ptrs.as_ptr();
        envp
    };
    sys_exec(path, argvp, envp)
}

/// Exit the current thread
pub fn exit(exit_code: usize) -> SysResult {
    sys_exit(exit_code)
}
