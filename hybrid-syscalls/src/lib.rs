//! 系统调用模块
#![no_std]
mod debug;
mod fs;
mod sync;
mod task;

extern crate alloc;

use alloc::sync::Arc;
use hal::user::UserInOutPtr;
use hybrid_objects::task::Thread;
use hybrid_objects::*;
use user_syscall::num::*;

/// 系统调用结构体，包含了用于执行一个系统调用的信息
pub struct Syscall<'a> {
    /// 执行系统调用的线程
    pub thread: &'a Arc<Thread>,
    /// Thread Function
    pub thread_fn: ThreadFn,
    // 新添加的系统调用编号字段
    pub syscall_num: u32,
    // 新添加的系统调用参数存储字段
    pub params: Vec<u8>,
}

impl Syscall<'_> {
    /// Get current process.
    pub fn process(&mut self) -> Arc<Process> {
        self.thread.proc().unwrap()
    }

    /// Get current thread.
    pub fn thread(&mut self) -> Arc<Thread> {
        self.thread.clone()
    }

    /// 系统调用总控函数
    pub async fn do_syscall(&mut self, syscall_id: usize, args: [usize; 6]) -> isize {
        #[allow(unused)]
        let [a0, a1, a2, a3, a4, a5] = args;
        let ret = match syscall_id {
            // 任务相关
            SYS_EXIT => self.sys_exit(a0),
            SYS_WAIT4 => self.sys_wait4(a0 as _, UserInOutPtr::from(a1)).await,
            SYS_SCHED_YIELD => self.sys_yield(),
            // Thread?
            SYS_GETPID => self.sys_get_pid(),
            SYS_GETTID => self.sys_get_tid(),
            SYS_FORK => self.sys_fork(),
            SYS_VFORK => self.sys_vfork(),
            SYS_EXECVE => self.sys_exec(args[0] as _, args[1] as _, args[2] as _),

            // 文件相关
            SYS_OPEN => self.sys_open(args[0] as _, args[1], args[2]),
            SYS_CLOSE => self.sys_close(args[0]),
            SYS_READ => self.sys_read(args[0], args[1], args[2]).await,
            SYS_WRITE => self.sys_write(args[0], args[1], args[2]).await,
            SYS_PIPE => self.sys_pipe(args[0] as _),
            // SYS_DUP => sys_dup(args[0]),
            _ => unimplemented!("Not implemented yet!"),
        };
        match ret {
            Ok(code) => code as _,
            Err(err) => -(err as isize),
        }
    }
}
