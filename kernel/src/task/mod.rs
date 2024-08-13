//! 任务管理模块
mod kthread;
mod process;
mod scheduler;
mod thread;

use alloc::sync::Arc;

pub use self::{kthread::*, process::*, scheduler::*, thread::*};
use crate::*;

/// 获取当前线程
#[inline]
pub fn current_thread() -> Arc<Thread> {
    CURRENT_THREAD.get().as_ref().unwrap().clone()
}

/// 获取当前进程
#[inline]
pub fn current_proc() -> Arc<Process> {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    let current_proc = current_thread.proc().unwrap();
    current_proc
}

/// 切换到pid所在进程的地址空间
pub fn activate_proc_ms(pid: usize) {
    let proc = PROCESS_MAP.get().get(&pid);
    proc.unwrap().memory_set().activate();
}
