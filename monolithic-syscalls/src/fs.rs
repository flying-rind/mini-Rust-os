//! 文件类系统调用

use crate::Syscall;
use alloc::vec;
use hal::user::UserOutPtr;
use user_syscall::SysResult;

impl Syscall<'_> {
    /// Write to a file descriptor
    pub fn sys_write(&mut self, fd: usize, buf: *const u8, size: usize) -> SysResult {
        // Debug
        // info!("fd = {}", fd);
        let mut proc = self.process();
        // FIXME: Should check first.
        let slice = unsafe { core::slice::from_raw_parts(buf, size) };
        let file = proc.get_file(fd)?;
        let len = file.write(slice);
        Ok(len as _)
    }

    /// Read from a file descriptor
    ///
    /// See [read(2)](https://man7.org/linux/man-pages/man2/read.2.html)
    pub async fn sys_read(&mut self, fd: usize, mut base: UserOutPtr<u8>, len: usize) -> SysResult {
        let mut proc = self.process();
        let file = proc.get_file(fd)?;
        let mut buf = vec![0u8; len];
        let len = file.read(&mut buf);
        let _ = base.write_array(&buf);
        Ok(len as _)
    }
}
