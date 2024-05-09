//! 管道抽象

use super::File;
use crate::*;
use alloc::{collections::VecDeque, rc::Weak};

/// 管道的一端
pub struct Pipe {
    writable: bool,
    buf: Rc<Cell<PipeBuffer>>,
}

/// 管道自身
pub struct PipeBuffer {
    buf: VecDeque<u8>,
    write_end: Weak<Pipe>,
}

/// 创建一个管道
///
/// 返回（读端，写端）的引用计数
pub fn make_pipe() -> (Rc<Pipe>, Rc<Pipe>) {
    let buf = Rc::new(Cell::new(PipeBuffer {
        buf: VecDeque::new(),
        write_end: Weak::new(),
    }));
    let write_end = Rc::new(Pipe {
        writable: true,
        buf: buf.clone(),
    });
    buf.get().write_end = Rc::downgrade(&write_end);
    let read_end = Rc::new(Pipe {
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

    /// 从管道的缓冲区读取
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
