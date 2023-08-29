//! 异步睡眠任务

use core::{
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
    task::{Context, Poll},
};
use futures_util::task::AtomicWaker;
use futures_util::Future;

/// 时钟中断计数
pub static TIMER_TICK: AtomicUsize = AtomicUsize::new(0);

/// 睡眠协程唤醒器
pub static TIMER_WAKER: AtomicWaker = AtomicWaker::new();

/// 时钟事件
pub fn timer_tick() {
    let ticks = TIMER_TICK.load(Ordering::Relaxed);
    if ticks > 0 {
        TIMER_TICK.fetch_sub(1, Ordering::Relaxed);
    }
    if ticks == 0 {
        TIMER_WAKER.wake();
    }
}

/// 睡眠协程
pub struct SleepFuture;

impl SleepFuture {
    /// 创建睡眠协程
    pub fn new(time: usize) -> Self {
        TIMER_TICK.store(time, Ordering::Relaxed);
        SleepFuture
    }
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if TIMER_TICK.load(Ordering::Relaxed) == 0 {
            Poll::Ready(())
        } else {
            TIMER_WAKER.register(&cx.waker());
            Poll::Pending
        }
    }
}

/// 异步睡眠
pub async fn sleep(time: usize, func: impl FnOnce()) {
    let sleep_future = SleepFuture::new(time);
    sleep_future.await;
    func();
}
