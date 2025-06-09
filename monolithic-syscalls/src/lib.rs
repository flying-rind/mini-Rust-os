//! 宏内核系统调用
#![no_std]

use alloc::{string::String, sync::Arc, vec::Vec};
use error::*;
use hybrid_objects::mm::PHYS_OFFSET;
use monolithic_objects::Thread;
use monolithic_objects::ThreadFn;
use num::*;
use num_derive::FromPrimitive;
use spin::MutexGuard;
use trapframe::UserContext;

pub use log::error;

extern crate alloc;
extern crate num_traits;

mod error;
mod fs;
mod num;
mod proc;

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
    /// Get current processs
    pub fn process(&self) -> MutexGuard<'_, monolithic_objects::Process> {
        self.thread.proc.lock()
    }

    /// 系统调用分发函数
    pub async fn syscall(&mut self, id: usize, args: [usize; 6]) -> isize {
        let [a0, a1, a2, a3, a4, a5] = args;
        let ret = match id {
            SYS_FORK => self.sys_fork(),
            SYS_VFORK => self.sys_vfork(),
            SYS_EXECVE => self.sys_exec(a0 as _, a1 as _, a2 as _),
            _ => unimplemented!("Not implemented yet"),
        };
        match ret {
            Ok(code) => code as _,
            Err(err) => -(err as isize),
        }
    }
}

/// 检查并复制C语言字符串
/// FIXME:Move to HAL
pub fn check_n_clone_cstr(user: *const u8) -> Result<String, SysError> {
    if user.is_null() {
        Ok(String::new())
    } else {
        let mut buffer = Vec::new();
        for i in 0.. {
            let addr = unsafe { user.add(i) };
            let data = copy_from_user(addr).ok_or(SysError::EFAULT)?;
            if data == 0 {
                break;
            }
            buffer.push(data);
        }
        String::from_utf8(buffer).map_err(|_| SysError::EFAULT)
    }
}

/// 检查并复制多个C字符串
/// FIXME:Move to HAL
pub fn check_n_clone_cstr_array(user: *const *const u8) -> Result<Vec<String>, SysError> {
    if user.is_null() {
        Ok(Vec::new())
    } else {
        let mut buffer = Vec::new();
        for i in 1.. {
            let addr = unsafe { user.add(i) };
            let str_ptr = copy_from_user(addr).ok_or(SysError::EFAULT)?;
            if str_ptr.is_null() {
                break;
            }
            let string = check_n_clone_cstr(str_ptr)?;
            buffer.push(string);
        }
        Ok(buffer)
    }
}

/// 从用户态复制到内核
/// FIXME:Move to HAL
pub fn copy_from_user<T>(addr: *const T) -> Option<T> {
    extern "C" fn read_user<T>(dst: *mut T, src: *const T) -> usize {
        unsafe {
            dst.copy_from_nonoverlapping(src, 1);
        }
        0
    }
    if !access_ok(addr as usize, size_of::<T>()) {
        return None;
    }
    let mut dst: T = unsafe { core::mem::zeroed() };
    match read_user(&mut dst, addr) {
        0 => Some(dst),
        _ => None,
    }
}

/// Check whether the address rage [addr, addr + len) is not in kernel space
/// FIXME:Move to HAL
pub fn access_ok(addr: usize, len: usize) -> bool {
    addr < PHYS_OFFSET && (addr + len) < PHYS_OFFSET
}
