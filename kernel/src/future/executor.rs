//! 协程执行器
use super::futures::*;
use crate::*;
use alloc::sync::Arc;
use core::{
    future::Future,
    task::{Context, Poll},
};
use spin::Lazy;
use woke::waker_ref;

/// 执行器状态
#[derive(Default, PartialEq, Eq, Clone)]
pub enum ExecutorState {
    #[default]
    /// 无任务运行
    Idle,
    /// 需要运行
    NeedRun,
}

/// 全局协程执行器
static EXECUTOR: Lazy<Cell<Arc<Executor>>> = Lazy::new(|| Cell::new(Arc::new(Executor::default())));

/// 协程执行器
#[derive(Default)]
pub struct Executor {
    /// 任务队列
    tasks_queue: Cell<VecDeque<Arc<Task>>>,
    /// 执行器状态
    state: Cell<ExecutorState>,
}

impl Executor {
    /// 添加任务
    pub fn add_task(&self, task: Arc<Task>) {
        self.tasks_queue.get_mut().push_back(task);
    }

    /// 轮讯所有就绪任务直到没有任务是就绪态
    pub fn run_until_idle(&self) {
        for _ in 0..self.tasks_queue.len() {
            let task = self.tasks_queue.get_mut().pop_front().unwrap();
            if task.need_poll() {
                // 每次轮讯都让其睡眠，等待唤醒后被再次轮讯或直接返回Ready
                task.sleep();
                // 由task创建waker
                let waker = waker_ref(&task);
                // 由waker创建context
                let mut context = Context::from_waker(&*waker);
                match task.poll_inner(&mut context) {
                    Poll::Ready(_) => continue,
                    Poll::Pending => self.tasks_queue.get_mut().push_back(task),
                }
            } else {
                self.tasks_queue.get_mut().push_back(task);
            }
        }
    }

    /// 设置执行器状态
    pub fn set_state(&self, state: ExecutorState) {
        *self.state.get_mut() = state;
    }

    /// 获取状态
    pub fn state(&self) -> ExecutorState {
        self.state.get().clone()
    }
}

/// 运行执行器直到没有就绪任务
pub fn run_util_idle() {
    EXECUTOR.get().set_state(ExecutorState::NeedRun);
    // 轮讯协程，直到任务队列中无就绪任务才停止
    EXECUTOR.run_until_idle();
    // 此时执行器任务队列中无就绪任务
    EXECUTOR.get().set_state(ExecutorState::Idle);
}

/// 是否需要调度执行
pub fn need_schedule() -> bool {
    EXECUTOR.get().state() == ExecutorState::NeedRun
}

/// 添加协程到执行器队列中
pub fn spawn(future: impl Future<Output = ()> + Send + Sync + 'static) {
    // 创建协程任务
    let weak_executor = Arc::downgrade(EXECUTOR.get());
    let task = Task::new(future, weak_executor);
    // 添加到执行器队列中
    EXECUTOR.get().add_task(task);
    // 协程执行器线程需要运行
    EXECUTOR.get().set_state(ExecutorState::NeedRun);
}
