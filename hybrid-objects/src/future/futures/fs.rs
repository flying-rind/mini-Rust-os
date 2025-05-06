//! 文件系统相关的协程

use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

use crate::fs::*;
use crate::Cell;

use alloc::sync::Arc;

use super::println;

/// 管道的读端等待管道写端关闭
pub struct WaitForPipeBuffer {
    /// 管道的缓冲区
    pipe_buffer: Arc<Cell<PipeBuffer>>,
}

impl WaitForPipeBuffer {
    /// 新建协程
    pub fn new(pipe_buffer: Arc<Cell<PipeBuffer>>) -> Self {
        WaitForPipeBuffer { pipe_buffer }
    }
}

impl Future for WaitForPipeBuffer {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.pipe_buffer.write_end().is_none() {
            // [Debug]
            println!("[Executor] WaitForPipeBuffer poll ready");
            // 写端已经被drop了，返回Ready
            Poll::Ready(())
        } else {
            // [Debug]
            println!("[Executor] WaitForPipeBuffer poll pending");
            // 写端还没退出，将唤醒器注册到写端中去
            let write_end = self.pipe_buffer.get().write_end();
            assert!(write_end.is_some());
            write_end.unwrap().add_waker(cx.waker().clone());
            Poll::Pending
        }
    }
}
