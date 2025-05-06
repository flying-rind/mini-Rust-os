//! 实现互斥锁
use crate::*;
use alloc::sync::Arc;
use core::task::Waker;
use future::{executor, futures::sync::WaitForMutex};

/// 基于阻塞和唤醒机制的互斥锁
#[derive(Default)]
pub struct MutexBlocking {
    /// 是否被上锁
    locked: Cell<bool>,
    // 等待锁的线程队列，以及他们的唤醒器
    wait_queue: Cell<VecDeque<(Arc<Thread>, Waker)>>,
}

impl MutexBlocking {
    /// 指定线程获得锁
    pub fn lock(&self, arc_self: Arc<MutexBlocking>, thread: Arc<Thread>) {
        if *self.locked {
            // 当前线程加入等待队列，并进入等待状态
            thread.set_state(ThreadState::Waiting);
            executor::spawn(WaitForMutex::new(thread, arc_self));
        } else {
            *self.locked.get_mut() = true;
            // println!("Thread {} get mutex now!", current_thread.tid());
        }
    }

    // 将自己等待队列中的第一个线程设置为就绪
    pub fn unlock(&self) {
        // 改变状态，这样协程才能够直到锁已释放，在协程中再重新上锁
        *self.locked.get_mut() = false;

        if let Some((_thread, waker)) = self.wait_queue.get_mut().pop_front() {
            // 唤醒第一个等待线程，他在协程轮讯时获得锁并上锁
            waker.wake_by_ref();
        }
    }

    /// 当前是否上锁
    pub fn is_locked(&self) -> bool {
        *self.locked
    }

    /// 将锁设为上锁状态
    pub fn set_locked(&self) {
        *self.locked.get_mut() = true;
    }

    /// 等待队列中添加一个线程唤醒器
    pub fn add_thread(&self, thread_waker: (Arc<Thread>, Waker)) {
        self.wait_queue.get_mut().push_back(thread_waker);
    }
}
