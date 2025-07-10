//! 任务管理模块
pub mod kthread;
mod process;
mod scheduler;
mod thread;

extern crate alloc;

use alloc::sync::Arc;

pub use self::{kthread::*, process::*, scheduler::*, thread::*};

/// 获取当前线程
#[inline]
pub fn current_thread() -> Arc<Thread> {
    CURRENT_THREAD.get().as_ref().unwrap().clone()
}

/// Set current thread.
pub fn set_current_thread(thread: Option<Arc<Thread>>) {
    *CURRENT_THREAD.get_mut() = thread;
}

/// 获取当前内核线程
#[inline]
pub fn current_kthread() -> Arc<Kthread> {
    CURRENT_KTHREAD.get().as_ref().unwrap().clone()
}

/// 获取当前进程
#[inline]
pub fn current_proc() -> Arc<Process> {
    let current_thread = current_thread();
    current_thread.proc().unwrap()
}

/// 切换到pid所在进程的地址空间
#[inline]
pub fn activate_proc_ms(pid: usize) {
    let proc = PROCESS_MAP.get().get(&pid);
    proc.unwrap().memory_set().activate();
}
