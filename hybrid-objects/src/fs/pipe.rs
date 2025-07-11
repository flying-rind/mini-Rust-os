//! 管道抽象

use crate::fs::File;
use crate::*;
use alloc::sync::Arc;
use alloc::sync::Weak;
use core::task::Waker;
use future::fs::WaitForPipeBuffer;
use hal::SysError;
use rcore_fs::vfs::FsError;
use user_syscall::SysResult;

/// 管道的一端
pub struct Pipe {
    /// 是否是写端
    writable: bool,
    /// 缓冲区
    buf: Arc<Cell<PipeBuffer>>,
    /// 写端持有读端的唤醒器，当写端被关闭时唤醒读端
    waker: Cell<Option<Waker>>,
}

/// 管道缓冲区
pub struct PipeBuffer {
    /// 缓冲区
    buf: Vec<u8>,
    /// 写端的一个弱引用
    write_end: Weak<File>,
}

impl PipeBuffer {
    /// 尝试获得写端的强引用
    pub fn write_end(&self) -> Option<Arc<File>> {
        self.write_end.upgrade()
    }
}

impl Pipe {
    /// 注册一个唤醒器，只能是写端
    pub fn add_waker(&self, waker: Waker) {
        let _ = self.waker.get_mut().insert(waker);
    }

    /// Readable
    pub fn readable(&self) -> bool {
        !self.writable
    }

    /// Writable
    pub fn writable(&self) -> bool {
        self.writable
    }

    /// 从管道的缓冲区读取到buf中
    ///
    /// 此时假设写端已经关闭，同步读取
    pub async fn read(&self, buf: &mut [u8]) -> SysResult {
        if !self.readable() {
            return Err(SysError::EPERM);
        }
        let wait4pipe = WaitForPipeBuffer::new(self.buf.clone());
        // Make sure the write end is cloesd already.
        wait4pipe.await;
        let pipe_buf = self.buf.buf.as_slice();
        let copy_len = buf.len().min(pipe_buf.len());
        let dst = &mut buf[..copy_len];
        dst.copy_from_slice(pipe_buf);
        Ok(copy_len)
    }

    /// 拓展管道的缓冲区
    pub fn write(&self, buf: &[u8]) -> SysResult {
        assert!(self.writable());
        self.buf.get_mut().buf.extend(buf.iter().copied());
        Ok(buf.len())
    }

    pub fn lookup_follow(
        &self,
        _path: &str,
        _max_follow: usize,
    ) -> rcore_fs::vfs::Result<Arc<dyn rcore_fs::vfs::INode>> {
        Err(FsError::NotFile)
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
pub fn make_pipe() -> (Arc<File>, Arc<File>) {
    let buf = Arc::new(Cell::new(PipeBuffer {
        buf: Vec::new(),
        write_end: Weak::new(),
    }));
    let write_end = Arc::new(File::Pipe(Pipe {
        writable: true,
        buf: buf.clone(),
        waker: Cell::new(None),
    }));
    buf.get_mut().write_end = Arc::downgrade(&write_end);
    let read_end = Arc::new(File::Pipe(Pipe {
        writable: false,
        buf: buf,
        waker: Cell::new(None),
    }));
    (read_end, write_end)
}
