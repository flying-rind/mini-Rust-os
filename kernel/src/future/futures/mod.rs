//! 定义异步系统调用需要的协程对象
pub mod fs;
pub mod sync;
pub mod task;

use core::task::{Context, Poll};
use core::{future::Future, pin::Pin};

use crate::*;
use alloc::sync::Arc;
use alloc::sync::Weak;
use future::executor::{Executor, ExecutorState};
pub use task::*;
use woke::Woke;

/// 协程执行器轮讯的协程任务
///
/// 包含一个Future对象和一个sleep标记
pub struct Task {
    /// 内含的协程
    inner_future: Cell<Pin<Box<dyn Future<Output = ()> + Send + Sync>>>,
    /// sleep标记，当为true时协程不会被执行器轮讯
    /// 协程的Waker和执行器executor是唯一能够改变
    /// sleep标记的代码区域，实现该Future的开发者
    /// 必须自行决定何时使用Waker来取消sleep标记
    sleep_flag: Cell<bool>,
    /// 执行器的若引用
    executor: Weak<Executor>,
}

impl Task {
    /// 将此任务休眠等待唤醒器唤醒
    ///
    /// 当轮讯任务返回阻塞时，Future应当保证将Waker注册到等待的事件中区
    pub fn sleep(&self) {
        *self.sleep_flag.get_mut() = true;
    }

    /// 唤醒任务
    pub fn wakeup(&self) {
        *self.sleep_flag.get_mut() = false;
    }

    /// 是否需要轮讯
    pub fn need_poll(&self) -> bool {
        !(*self.sleep_flag.get())
    }

    /// 轮讯内含的协程
    pub fn poll_inner(&self, context: &mut Context<'_>) -> Poll<()> {
        self.inner_future.get_mut().as_mut().poll(context)
    }

    /// 新建一个task
    pub fn new(
        future: impl Future<Output = ()> + Send + Sync + 'static,
        executor: Weak<Executor>,
    ) -> Arc<Self> {
        Arc::new(Task {
            inner_future: Cell::new(Box::pin(future)),
            sleep_flag: Cell::new(false),
            executor,
        })
    }
}

impl Woke for Task {
    /// 唤醒任务，且将执行器设置为需要执行
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.wakeup();
        arc_self
            .executor
            .upgrade()
            .unwrap()
            .set_state(ExecutorState::NeedRun);
    }
}
