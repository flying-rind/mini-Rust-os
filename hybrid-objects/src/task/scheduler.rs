//! 线程总调度器
use super::*;
use alloc::sync::Arc;

/// 全局线程调度器
///
/// 统一管理内核线程和用户线程
pub struct Scheduler {}

impl Scheduler {
    /// 获取第一个需要运行的内核线程，
    ///
    /// 将其弹出全局队列，应当确保当前内核线程不在当前全局队列中
    pub fn get_first_kthread() -> Option<Arc<Kthread>> {
        let kthread_deque = KTHREAD_DEQUE.get_mut();
        for _ in 0..kthread_deque.len() {
            let kthread = kthread_deque.pop_front().unwrap();
            if kthread.need_schedule() {
                return Some(kthread.clone());
            } else {
                kthread_deque.push_back(kthread);
            }
        }
        None
    }

    /// 获取第一个需要运行的用户线程
    ///
    /// 若没有则返回None
    pub fn get_first_uthread() -> Option<Arc<Thread>> {
        let thread_deque = THREADS_DEQUE.get_mut();
        for _ in 0..thread_deque.len() {
            let thread = thread_deque.pop_front().unwrap();
            if thread.state() == ThreadState::Runnable {
                return Some(thread.clone());
            } else {
                thread_deque.push_back(thread);
            }
        }
        None
    }

    /// 当前内核线程放弃CPU，调度下一个就绪内核线程
    pub fn yield_current_kthread() {
        let kthread = Scheduler::get_first_kthread();
        if let Some(kthread) = kthread {
            let current_kthread = current_kthread();
            current_kthread.switch_to(current_kthread.clone(), kthread);
        }
    }
}
