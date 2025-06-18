//! 文件类系统调用

use log::info;
use user_syscall::SysResult;

use crate::Syscall;

impl Syscall<'_> {
    /// Write to a file descriptor
    pub fn sys_write(&mut self, fd: usize, buf: *const u8, size: usize) -> SysResult {
        // Debug
        info!("fd = {}", fd);
        let mut proc = self.process();
        // FIXME: Should check first.
        let slice = unsafe { core::slice::from_raw_parts(buf, size) };
        let file = proc.get_file(fd)?;
        let len = file.write(slice);
        Ok(len)
    }
}
