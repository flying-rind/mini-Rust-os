//! 管道抽象

use super::File;
use crate::future::executor;
use crate::*;
use alloc::sync::Arc;
use alloc::sync::Weak;
use core::task::Waker;
use future::futures::fs::WaitForPipeBuffer;

/// 管道的一端
pub struct Pipe {
    /// 是否是写端
    writable: bool,
    /// 缓冲区
    buf: Arc<Cell<PipeBuffer>>,
    /// 写端持有读端的唤醒器，当写端被关闭时唤醒读端
    waker: Cell<Option<Waker>>,
}

impl Pipe {
    /// 注册一个唤醒器，只能是写端
    pub fn add_waker(&self, waker: Waker) {
        let _ = self.waker.get_mut().insert(waker);
    }

    /// 异步读取管道，将读取的字节写回用户态
    pub fn async_read(
        &self,
        pipe_end: Arc<Pipe>,
        buf_ptr: usize,
        buf_len: usize,
        result_ptr: usize,
    ) {
        // 等待写端关闭，读入到buf中
        let current_thread = current_thread();
        // 进入等待
        current_thread.set_state(ThreadState::Waiting);
        // 生成协程
        executor::spawn(wait_for_pipe_and_read(
            pipe_end,
            self.buf.clone(),
            current_thread.clone(),
            buf_ptr,
            buf_len,
            result_ptr,
        ));
    }
}

/// 等待写端关闭后读入buf且将读取字节数写入用户态
async fn wait_for_pipe_and_read(
    pipe_end: Arc<Pipe>,
    pipe_buffer: Arc<Cell<PipeBuffer>>,
    thread: Arc<Thread>,
    buf_ptr: usize,
    buf_len: usize,
    result_ptr: usize,
) {
    let wait_for_pipebuffer = WaitForPipeBuffer::new(pipe_buffer);
    // 确保写端已经关闭
    wait_for_pipebuffer.await;
    thread.proc().unwrap().memory_set().activate();
    // 解析buf
    let buf_ptr = buf_ptr as *mut u8;
    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, buf_len) };
    let read_size = pipe_end.read(buf);
    // println!("result_ptr: {:x}, read_size: {}", result_ptr, read_size);
    // 写回用户态
    let result_ptr = result_ptr as *mut usize;
    unsafe { *result_ptr = read_size }
    // 等待线程就绪
    thread.set_state(ThreadState::Runnable);
}

/// 管道缓冲区
pub struct PipeBuffer {
    /// 缓冲区
    buf: Vec<u8>,
    /// 写端的一个弱引用
    write_end: Weak<Pipe>,
}

impl PipeBuffer {
    /// 尝试获得写端的强引用
    pub fn write_end(&self) -> Option<Arc<Pipe>> {
        self.write_end.upgrade()
    }
}

impl Drop for Pipe {
    // 写端析构时唤醒阻塞的读端
    fn drop(&mut self) {
        if self.writable {
            let waker = self.waker.get().as_ref();
            if let Some(waker) = waker {
                waker.wake_by_ref();
            }
        }
    }
}

/// 创建一个管道
///
/// 返回（读端，写端）的引用
pub fn make_pipe() -> (Arc<Pipe>, Arc<Pipe>) {
    let buf = Arc::new(Cell::new(PipeBuffer {
        buf: Vec::new(),
        write_end: Weak::new(),
    }));
    let write_end = Arc::new(Pipe {
        writable: true,
        buf: buf.clone(),
        waker: Cell::new(None),
    });
    buf.get_mut().write_end = Arc::downgrade(&write_end);
    let read_end = Arc::new(Pipe {
        writable: false,
        buf: buf,
        waker: Cell::new(None),
    });
    (read_end, write_end)
}

impl File for Pipe {
    fn readable(&self) -> bool {
        !self.writable
    }

    fn writable(&self) -> bool {
        self.writable
    }

    /// 从管道的缓冲区读取到buf中
    ///
    /// 此时假设写端已经关闭，同步读取
    fn read(&self, buf: &mut [u8]) -> usize {
        assert!(self.readable());
        let pipe_buf = self.buf.buf.as_slice();
        let copy_len = buf.len().min(pipe_buf.len());
        // println!(
        //     "pipe_buf len: {}; buf len: {}, copy_len: {}",
        //     pipe_buf.len(),
        //     buf.len(),
        //     copy_len
        // );
        let dst = &mut buf[..copy_len];
        dst.copy_from_slice(pipe_buf);
        // println!("dst len: {}", dst.len());
        copy_len
    }

    /// 拓展管道的缓冲区
    fn write(&self, buf: &[u8]) -> usize {
        assert!(self.writable());
        self.buf.get_mut().buf.extend(buf.iter().copied());
        buf.len()
    }
}
