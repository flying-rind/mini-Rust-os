//! 任务管理类系统调用
use crate::monolithic::error::SysError;
use crate::monolithic::error::SysResult;
use alloc::string::String;
use alloc::vec::Vec;
use num_traits::FromPrimitive;

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
pub fn exec(path: &str, argv: Vec<String>, env: Vec<String>) -> SysResult {
    let path = path.as_ptr() as *const u8;
    let argvp = argv.as_ptr() as *const *const u8;
    let envp = env.as_ptr() as *const *const u8;
    sys_exec(path, argvp, envp)
}

/// Exit the current thread
pub fn exit(exit_code: usize) -> SysResult {
    sys_exit(exit_code)
}
