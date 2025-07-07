//! 系统调用模块
#![no_std]
mod debug;
mod fs;
mod sync;
mod task;

extern crate alloc;

use alloc::sync::Arc;
use fs::*;
use hybrid_objects::task::Thread;
use hybrid_objects::*;
use task::*;
use user_syscall::num::*;

/// 系统调用结构体，包含了用于执行一个系统调用的信息
pub struct Syscall<'a> {
    /// 执行系统调用的线程
    pub thread: &'a Arc<Thread>,
}

impl Syscall<'_> {
    /// 系统调用总控函数
    pub fn do_syscall(&mut self, syscall_id: usize, args: [usize; 6]) -> isize {
        let ret = match syscall_id {
            // 调试用
            // DebugWrite => sys_debug_write(args[0]),
            // DebugDataTransport => sys_debug_data_transport(args[0], args[1]),
            // DebugOpen => sys_debug_open(args[0]),
            // SerialRead => sys_serial_read(args[0]),
            // GetTime => (*pic::TICKS as _, 0),
            // TestCstr => sys_test_cstr(args[0] as _),

            // 任务相关
            SYS_EXIT => sys_exit(args[0]),
            SYS_WAIT4 => sys_proc_wait(args[0]),
            // ?
            SYS_SCHED_YIELD => sys_yield(),
            // Thread?
            SYS_GETPID => sys_get_pid(),
            SYS_GETTID => sys_get_tid(),
            SYS_FORK => sys_fork(),
            SYS_VFORK => sys_fork(),
            SYS_EXECVE => sys_exec(args[0], args[1]),

            // 文件相关
            SYS_OPEN => sys_open(args[0], args[1], args[2]),
            SYS_CLOSE => sys_close(args[0]),
            SYS_READ => sys_read(args[0], args[1], args[2], args[3]),
            SYS_WRITE => sys_write(args[0], args[1], args[2], args[3]),
            SYS_PIPE => sys_pipe(),
            SYS_DUP => sys_dup(args[0]),

            _ => unimplemented!("Not implemented yet!"),
        };
        match ret {
            Ok(code) => code as _,
            Err(err) => -(err as isize),
        }
    }
}
