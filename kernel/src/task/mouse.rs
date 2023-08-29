//! 鼠标事件流

use crate::driver::mouse::mouse_event;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::stream::Stream;
use futures_util::stream::StreamExt;
use futures_util::task::AtomicWaker;
use spin::Once;

/// 鼠标事件队列
static PACKAGE_QUEUE: Once<ArrayQueue<u8>> = Once::new();

/// 鼠标协程唤醒器
static MOUSE_WAKER: AtomicWaker = AtomicWaker::new();

/// 添加鼠标事件
pub fn add_package(package: u8) {
    if let Some(queue) = PACKAGE_QUEUE.get() {
        if let Err(_) = queue.push(package) {
            serial_println!("WARNING: package queue full; dropping mouse input");
        } else {
            MOUSE_WAKER.wake();
        }
    } else {
        serial_println!("WARNING: package queue uninitialized");
    }
}

/// 异步事件流
pub struct PackageStream;

impl PackageStream {
    /// 创建鼠标事件流
    pub fn new() -> Self {
        PACKAGE_QUEUE.call_once(|| ArrayQueue::new(100));
        PackageStream
    }
}

impl Stream for PackageStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = PACKAGE_QUEUE.get().expect("package queue not initialized");

        if let Some(package) = queue.pop() {
            return Poll::Ready(Some(package));
        }

        MOUSE_WAKER.register(&cx.waker());
        match queue.pop() {
            Some(package) => {
                MOUSE_WAKER.take();
                Poll::Ready(Some(package))
            }
            None => Poll::Pending,
        }
    }
}

/// 异步任务
pub async fn print_mousemovements() {
    let mut packages = PackageStream::new();
    while let Some(package) = packages.next().await {
        mouse_event(package);
    }
}
