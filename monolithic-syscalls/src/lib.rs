//! 宏内核系统调用
#![no_std]

use crate::fs::Stat;
use alloc::boxed::Box;
use alloc::{sync::Arc, vec::Vec};
use hal::SysError;
use hal::user::{UserInOutPtr, UserPtr};
pub use log::error;
use log::{info, warn};
use monolithic_objects::Thread;
use monolithic_objects::ThreadFn;
use monolithic_objects::fs::iovec::IoVec;
use num::*;
use spin::MutexGuard;
use trapframe::UserContext;
use user_syscall::SysResult;

extern crate alloc;
extern crate num_traits;

mod custom;
mod fs;
mod mem;
mod misc;
mod num;
mod proc;
mod signal;
mod sync;
mod system;

/// 系统调用
pub struct Syscall<'a> {
    /// 系统调用的用户线程
    pub thread: &'a Arc<Thread>,
    /// 用户线程线程函数
    pub thread_fn: ThreadFn,
    /// User-space context, we will put it into thread after trap/syscall.
    pub context: &'a mut Box<UserContext>,
}

impl Syscall<'_> {
    /// Get current processs
    pub fn process(&self) -> MutexGuard<'_, monolithic_objects::Process> {
        self.thread.proc.lock()
    }

    /// Get current thread
    pub fn thread(&mut self) -> Arc<Thread> {
        self.thread.clone()
    }

    /// 系统调用分发函数
    pub async fn syscall(&mut self, id: usize, args: [usize; 6]) -> isize {
        #[allow(unused)]
        let [a0, a1, a2, a3, a4, a5] = args;
        let ret = match id {
            // TASK
            SYS_FORK => self.sys_fork(),
            SYS_VFORK => self.sys_vfork(),
            SYS_EXECVE => self.sys_exec(a0 as _, a1 as _, a2 as _),
            SYS_EXIT => self.sys_exit(a0 as _),
            SYS_WAIT4 => self.sys_wait4(a0 as _, UserInOutPtr::from(a1)).await,
            SYS_EXIT_GROUP => self.sys_exit_group(a0),
            SYS_SET_TID_ADDRESS => self.sys_set_tid_address(a0 as _),
            SYS_GETUID => self.unimplemented("getuid", Ok(0)),
            SYS_GETGID => self.unimplemented("getgid", Ok(0)),
            SYS_SETUID => self.unimplemented("setuid", Ok(0)),
            SYS_GETEUID => self.unimplemented("geteuid", Ok(1000)),
            SYS_GETEGID => self.unimplemented("getegid", Ok(0)),
            SYS_GETPID => self.sys_getpid(),
            SYS_GETPPID => self.sys_getppid(),

            // System.
            SYS_UNAME => self.sys_uname(a0 as _),

            // FS
            SYS_WRITE => self.sys_write(a0 as _, a1 as _, a2 as _),
            SYS_READ => self.sys_read(a0.into(), a1.into(), a2 as _).await,
            SYS_DUP2 => self.sys_dup2(a0, a1),
            SYS_DUP3 => self.sys_dup3(a0, a1, a2),
            SYS_FCNTL => self.sys_fcntl(a0, a1, a2),
            SYS_IOCTL => self.sys_ioctl(a0, a1, a2, a3, a4),
            SYS_WRITEV => self.sys_writev(a0, a1 as *const IoVec, a2),
            SYS_OPEN => self.sys_open(a0 as _, a1, a2),
            SYS_OPENAT => self.sys_openat(a0, a1 as _, a2, a3),
            SYS_CLOSE => self.sys_close(a0),
            SYS_STAT => self.sys_stat(a0 as *const u8, a1 as *mut Stat),
            SYS_FSTAT => self.sys_fstatat(a0 as _, a1 as _, a2 as _, a3),
            SYS_GETCWD => self.sys_getcwd(a0 as _, a1),

            // Signal
            SYS_RT_SIGRETURN => self.sys_rt_sigreturn(),
            SYS_RT_SIGPROCMASK => self.sys_rt_procmask(a0, a1.into(), a2.into(), a3),
            SYS_RT_SIGACTION => self.sys_rt_sigaction(a0, a1.into(), a2.into(), a3),

            // Mem
            SYS_BRK => self.unimplemented("brk", Err(SysError::ENOMEM)),
            SYS_MMAP => self.sys_mmap(a0, a1, a2, a3, a4, a5),
            SYS_MUNMAP => self.sys_munmap(a0, a1),
            SYS_MPROTECT => self.sys_mprotect(a0, a1, a2),

            // Sync
            SYS_FUTEX => {
                self.sys_futex(a0, a1 as _, a2 as _, UserPtr::from(a3))
                    .await
            }

            // MISC
            SYS_ARCH_PRCTL => self.sys_arch_prctl(a0 as _, a1),

            // Custom
            SYS_TEST_CSTR => self.sys_test_cstr(a0 as _),
            _ => unimplemented!("syscall id {} Not implemented yet", id),
        };
        match ret {
            Ok(code) => code as _,
            Err(err) => -(err as isize),
        }
    }

    /// Mark unimplemented.
    fn unimplemented(&self, name: &str, ret: SysResult) -> SysResult {
        warn!("{} is unimplemented", name);
        ret
    }
}
