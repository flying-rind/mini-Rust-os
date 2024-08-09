//! 条件变量

use crate::{
    future::{executor, futures::sync::WaitForCondvar},
    task::*,
    Cell,
};
use alloc::{collections::vec_deque::VecDeque, sync::Arc};
use core::task::Waker;

use crate::task::Thread;

use super::MutexBlocking;

/// 条件变量
#[derive(Default)]
pub struct Condvar {
    // 阻塞的线程队列和他们的唤醒器
    wait_queue: Cell<VecDeque<(Arc<Thread>, Waker)>>,
    /// 阻塞线程的睡眠标记，true表示仍在阻塞
    /// 唤醒时置为false，以便通知协程执行器
    /// 协程已经准备好了，这个队列在执行器中
    /// 被弹出
    sleep_flags: Cell<VecDeque<bool>>,
}

impl Condvar {
    /// 新建条件变量
    pub fn new() -> Arc<Self> {
        Arc::new(Condvar::default())
    }

    /// 唤醒一个等待线程
    pub fn signal(&self) {
        if let Some((_thread, waker)) = self.wait_queue.get_mut().pop_front() {
            // 唤醒第一个等待线程对应的协程
            self.sleep_flags.get_mut()[0] = false;
            waker.wake_by_ref();
        }
    }

    /// 当前线程阻塞直到被条件变量唤醒并重新获得锁
    pub fn wait(&self, mutex: Arc<MutexBlocking>, arc_self: Arc<Condvar>) {
        let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
        // 释放锁
        mutex.unlock();
        // 阻塞当前线程并异步等待
        current_thread.set_state(ThreadState::Waiting);
        // 创建等待协程
        executor::spawn(wait_for_condvar_then_lock(current_thread, arc_self, mutex));
    }

    /// 查讯自己的第一个阻塞线程睡眠标记
    pub fn first_sleep_flag(&self) -> bool {
        if self.sleep_flags.get().len() == 0 {
            return false;
        }
        self.sleep_flags.get()[0]
    }

    /// 弹出首个sleep_flag，线程唤醒时在协程中调用
    pub fn pop_sleep_flag(&self) {
        self.sleep_flags.get_mut().pop_front();
    }

    /// 添加一个阻塞线程
    pub fn add_thread(&self, (thread, waker): (Arc<Thread>, Waker)) {
        self.wait_queue.get_mut().push_back((thread, waker));
        // 添加睡眠标记
        self.sleep_flags.get_mut().push_back(true);
    }
}

/// 等待条件变量唤醒后还需要等待锁，可能会创建两个协程
/// 因此使用await语法来保证两个协程的时序
async fn wait_for_condvar_then_lock(
    thread: Arc<Thread>,
    condvar: Arc<Condvar>,
    mutex: Arc<MutexBlocking>,
) {
    let wait_condvar = WaitForCondvar::new(thread.clone(), condvar);
    // 确保首先被信号量唤醒
    wait_condvar.await;
    // 再获得锁
    mutex.lock(mutex.clone(), thread);
}
