//! 任务相关的协程结构体

use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

use super::println;
use super::trap::Kthread;
use super::trap::ThreadState;
use super::trap::{Process, Thread};

/// 用户线程等待一个进程结束
///
/// 这个协程是顶层协程，直接被执行器轮讯，需要保证在返回Ready前将等待中线程状态设置为Runnable
pub struct WaitForProc {
    /// 正在等待的线程
    thread: Arc<Thread>,
    /// 被等待的进程
    waited_process: Arc<Process>,
}

impl WaitForProc {
    /// 新建协程
    pub fn new(thread: Arc<Thread>, waited_process: Arc<Process>) -> Self {
        WaitForProc {
            thread,
            waited_process,
        }
    }
}

impl Future for WaitForProc {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 等待其根线程结束
        let waited_root_thread = self.waited_process.root_thread();
        if waited_root_thread.state() == ThreadState::Exited {
            // 已经退出，将等待的线程设置为就绪态
            self.thread.set_state(ThreadState::Runnable);
            Poll::Ready(())
        } else {
            // 向被等待线程添加一个唤醒器，其状态改变时再唤醒这个协程
            waited_root_thread.add_state_waker(cx.waker().clone(), ThreadState::Exited);
            Poll::Pending
        }
    }
}

/// 当前线程等待一个内核线程完成某个请求
///
/// 这个协程是顶层协程，直接被执行器轮讯，需要保证在返回Ready前将等待中线程状态设置为Runnable
pub struct WaitForKthread {
    /// 等待的线程
    thread: Arc<Thread>,
    /// 被等待的内核线程
    kthread: Arc<Kthread>,
    /// 请求ID
    req_id: usize,
}

impl WaitForKthread {
    /// 新建协程
    pub fn new(thread: Arc<Thread>, kthread: Arc<Kthread>, req_id: usize) -> Self {
        WaitForKthread {
            thread,
            kthread,
            req_id,
        }
    }
}

impl Future for WaitForKthread {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 已完成这个服务，返回Ready，且将等待线程唤醒
        if self.kthread.response_id() >= self.req_id {
            self.thread.set_state(ThreadState::Runnable);
            return Poll::Ready(());
        } else {
            println!(
                "\x1b[33m[Executor] WaitForKthread poll pending, response_id: {}, req_id: {}\x1b[0m",
                self.kthread.response_id(),
                self.req_id
            );
            // 服务未完成，将唤醒器注册到内核线程中去
            self.kthread.add_waker(cx.waker().clone(), self.req_id);
            Poll::Pending
        }
    }
}

/// 线程主动放弃CPU直到下一次被调度
pub struct ThreadYield {
    /// 等待的线程
    thread: Arc<Thread>,
    // 是否准备好
    ready_flag: bool,
}

impl ThreadYield {
    pub fn new(thread: Arc<Thread>) -> Self {
        Self {
            thread,
            ready_flag: false,
        }
    }
}

impl Future for ThreadYield {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.ready_flag == true {
            // 已经退出，将等待的线程设置为就绪态
            self.thread.set_state(ThreadState::Runnable);
            Poll::Ready(())
        } else {
            // 自己唤醒自己，下一次被执行器轮讯时就会返回就绪
            self.ready_flag = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// 等待另一个线程
pub struct WaitForThread {
    /// 正在等待的线程
    waiting_thread: Arc<Thread>,
    /// 被等待的线程
    waited_thread: Arc<Thread>,
}

impl WaitForThread {
    pub fn new(waiting_thread: Arc<Thread>, waited_thread: Arc<Thread>) -> Self {
        Self {
            waiting_thread,
            waited_thread,
        }
    }
}

impl Future for WaitForThread {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 等待的线程已结束
        if self.waited_thread.state() == ThreadState::Exited {
            // 已经退出，将等待的线程设置为就绪态
            self.waiting_thread.set_state(ThreadState::Runnable);
            Poll::Ready(())
        } else {
            // 向被等待线程添加一个唤醒器，其状态改变时再唤醒这个协程
            self.waited_thread
                .add_state_waker(cx.waker().clone(), ThreadState::Exited);
            Poll::Pending
        }
    }
}
