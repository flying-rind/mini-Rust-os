//! 协程执行器
use crate::*;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::{
    future::Future,
    task::{Context, Poll},
};
use lazy_static::lazy_static;
use spin::Mutex;
use woke::waker_ref;

lazy_static! {
    /// 全局协程执行器
    static ref GLOBAL_EXECUTOR: Executor = Executor::default();
}

/// 协程执行器
#[derive(Default)]
pub struct Executor {
    /// 任务队列
    tasks_queue: Mutex<VecDeque<Arc<Task>>>,
}

impl Executor {
    /// 添加任务
    pub fn add_task(&self, task: Arc<Task>) {
        self.tasks_queue.lock().push_back(task);
    }

    /// Get first runnable task
    pub fn pop_runnable_task(&self) -> Option<Arc<Task>> {
        let mut tasks = self.tasks_queue.lock();
        for _ in 0..tasks.len() {
            let task = tasks.pop_front().unwrap();
            if task.need_poll() {
                return Some(task);
            } else {
                tasks.push_back(task);
            }
        }
        None
    }

    /// 轮讯所有就绪任务直到没有任务是就绪态
    pub fn run_until_idle(&self) {
        while let Some(task) = self.pop_runnable_task() {
            // 每次轮讯都让其睡眠，等待唤醒后被再次轮讯或直接返回Ready
            task.sleep();
            // 由task创建waker
            let waker = waker_ref(&task);
            // 由waker创建context
            let mut context = Context::from_waker(&*waker);
            match task.poll_inner(&mut context) {
                Poll::Ready(_) => continue,
                Poll::Pending => self.add_task(task),
            }
        }
    }
}

/// 运行执行器直到没有就绪任务
pub fn run_util_idle() {
    // 轮讯协程，直到任务队列中无就绪任务才停止
    GLOBAL_EXECUTOR.run_until_idle();
}

/// 添加协程到执行器队列中
pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    // 创建协程任务
    let task = Task::new(future);
    // 添加到执行器队列中
    GLOBAL_EXECUTOR.add_task(task);
}
