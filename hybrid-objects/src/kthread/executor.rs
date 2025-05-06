//! 内核线程协程执行器

use crate::future::executor;
use crate::task::Scheduler;

/// 协程执行器内核线程入口
pub fn executor_entry(_ktid: usize) {
    // 无限循环运行内核协程
    loop {
        executor::run_util_idle();
        Scheduler::yield_current_kthread();
    }
}
