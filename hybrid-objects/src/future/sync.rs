//! 互斥工具模块使用的协程

use crate::sync::*;
use crate::task::*;

use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

// use super::println;

/// 线程等待一个锁
pub struct WaitForMutex {
    /// 正在等待的线程
    thread: Arc<Thread>,
    /// 被等待的锁
    mutex: Arc<MutexBlocking>,
}

impl WaitForMutex {
    /// 新建协程
    pub fn new(thread: Arc<Thread>, mutex: Arc<MutexBlocking>) -> Self {
        Self { thread, mutex }
    }
}

impl Future for WaitForMutex {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.mutex.is_locked() {
            // 锁已经释放，线程获得锁且恢复为就绪态
            self.thread.set_state(ThreadState::Runnable);
            self.mutex.set_locked();
            // [Debug]
            // println!("thread: {:x} get mutex now!", self.thread.tid());
            Poll::Ready(())
        } else {
            // 锁被占用了，将线程和唤醒器注册到锁里面去
            self.mutex
                .add_thread((self.thread.clone(), cx.waker().clone()));
            Poll::Pending
        }
    }
}

/// 线程等待一个信号量
pub struct WaitForSem {
    /// 等待的线程
    thread: Arc<Thread>,
    /// 被等待的信号量
    sem: Arc<Sem>,
}

impl WaitForSem {
    /// 新建协程
    pub fn new(thread: Arc<Thread>, sem: Arc<Sem>) -> Self {
        Self { thread, sem }
    }
}

impl Future for WaitForSem {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if *(self.sem.n.get()) >= 1 {
            // 有空闲资源
            self.thread.set_state(ThreadState::Runnable);
            *(self.sem.n.get_mut()) -= 1;
            Poll::Ready(())
        } else {
            // 没有资源，将线程和唤醒器注册到信号量阻塞队列里面去
            self.sem
                .add_thread((self.thread.clone(), cx.waker().clone()));
            Poll::Pending
        }
    }
}

/// 线程等待一个条件变量
pub struct WaitForCondvar {
    /// 等待的线程
    thread: Arc<Thread>,
    /// 被等待的条件变量
    condvar: Arc<Condvar>,
}

impl WaitForCondvar {
    /// 新建协程
    pub fn new(thread: Arc<Thread>, condvar: Arc<Condvar>) -> Self {
        Self { thread, condvar }
    }
}

impl Future for WaitForCondvar {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.condvar.first_sleep_flag() == false {
            // 将第一个sleep_flag弹出
            self.condvar.pop_sleep_flag();
            // 已被唤醒
            self.thread.set_state(ThreadState::Runnable);
            Poll::Ready(())
        } else {
            // 没有资源，将线程和唤醒器注册到信号量阻塞队列里面去
            self.condvar
                .add_thread((self.thread.clone(), cx.waker().clone()));
            Poll::Pending
        }
    }
}
