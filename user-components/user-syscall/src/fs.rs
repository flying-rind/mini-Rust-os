//! 文件系统类系统调用

use crate::{sys_close, sys_dup, sys_open, sys_read, sys_write, SysResult};

bitflags::bitflags! {
    /// 打开文件时的读写权限
    pub struct OpenFlags: usize {
        /// read only
        const RDONLY = 0;
        /// write only
        const WRONLY = 1 << 0;
        /// read write
        const RDWR = 1 << 1;
        /// create file if it does not exist
        const CREATE = 1 << 6;
        /// error if create and the file exists
        const EXCLUSIVE = 1 << 7;
        /// truncate file upon open
        const TRUNCATE = 1 << 9;
        /// append on each write
        const APPEND = 1<<10;
        /// close on exec
        const CLOEXEC = 1 << 19;
    }
}
/// Write to file descriptor
pub fn write(fd: usize, buf: &[u8]) -> SysResult {
    let buf_ptr = buf.as_ptr() as *const u8;
    let size = buf.len();
    sys_write(fd, buf_ptr, size)
}

/// Read from a file descriptor
pub fn read(fd: usize, buf: &mut [u8]) -> SysResult {
    let buf_ptr = buf.as_mut_ptr();
    let size = buf.len();
    sys_read(fd, buf_ptr, size)
}

/// Close a file.
pub fn close(fd: usize) -> SysResult {
    sys_close(fd)
}

/// Open a file.
pub fn open(path: &str, flags: usize) -> SysResult {
    sys_open(path.as_ptr(), flags, 0)
}

/// Dup
pub fn dup(arg1: usize) -> SysResult {
    sys_dup(arg1)
}
