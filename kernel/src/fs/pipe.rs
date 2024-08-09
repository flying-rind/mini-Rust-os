//! 管道抽象

use super::File;
use crate::*;
use alloc::sync::Arc;
use alloc::{collections::VecDeque, sync::Weak};

/// 管道的一端
pub struct Pipe {
    /// 是否是写端
    writable: bool,
    /// 缓冲区
    buf: Arc<Cell<PipeBuffer>>,
}

/// 管道缓冲区
pub struct PipeBuffer {
    /// 缓冲区
    buf: VecDeque<u8>,
    /// 写端的一个弱引用
    write_end: Weak<Pipe>,
}

/// 创建一个管道
///
/// 返回（读端，写端）的引用
pub fn make_pipe() -> (Arc<Pipe>, Arc<Pipe>) {
    let buf = Arc::new(Cell::new(PipeBuffer {
        buf: VecDeque::new(),
        write_end: Weak::new(),
    }));
    let write_end = Arc::new(Pipe {
        writable: true,
        buf: buf.clone(),
    });
    buf.get().write_end = Arc::downgrade(&write_end);
    let read_end = Arc::new(Pipe {
        writable: false,
        buf: buf,
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
    fn read(&self, buf: &mut [u8]) -> usize {
        assert!(self.readable());
        let mut buf = buf.into_iter();
        let mut n = 0;
        let pipe_buf = self.buf.get();
        loop {
            if pipe_buf.buf.is_empty() {
                // 管道对应的所有写端都已关闭
                if pipe_buf.write_end.upgrade().is_none() {
                    return n;
                }
                // 尚不能读取，当前线程主动放弃CPU
                task::current_yield();
            }
            // 将管道中的字节读出写入buf中
            while let Some(&x) = pipe_buf.buf.front() {
                if let Some(b) = buf.next() {
                    *b = x;
                    pipe_buf.buf.pop_front();
                    n += 1;
                } else {
                    return n;
                }
            }
        }
    }

    /// 拓展管道的缓冲区
    fn write(&self, buf: &[u8]) -> usize {
        assert!(self.writable());
        self.buf.get().buf.extend(buf.iter().copied());
        buf.len()
    }
}
