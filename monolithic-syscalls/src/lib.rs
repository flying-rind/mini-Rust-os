//! 宏内核系统调用
#![no_std]

use alloc::sync::Arc;
use monolithic_objects::Thread;
use num_derive::FromPrimitive;
use trapframe::UserContext;
use monolithic_objects::ThreadFn;
use error::*;
use num::*;

extern crate alloc;
extern crate num_traits;

mod proc;
mod error;
mod num;

pub type SysResult = Result<usize, SysError>;

/// 系统调用
pub struct Syscall<'a> {
    /// 系统调用的用户线程
    pub thread: &'a Arc<Thread>,
    /// 用户态上下文
    pub context: &'a mut UserContext,
    /// 用户线程线程函数
    pub thread_fn: ThreadFn,

}

impl Syscall<'_> {
    /// 系统调用分发函数
    pub async fn syscall(&mut self, id: usize, args: [usize; 6]) -> isize {
        let [a0, a1, a2, a3, a4, a5] = args;
        let ret = match id {
            SYS_FORK => self.sys_fork(),
            _ => unimplemented!("Not implemented"),
        };
        match ret {
            Ok(code) => code as _,
            Err(err) => -(err as isize),
        }
    }
}

