//! 文件系统类系统调用

use crate::{sys_dup, sys_read, sys_write, SysResult};

bitflags::bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
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

/// Dup
pub fn dup(arg1: usize) -> SysResult {
    sys_dup(arg1)
}
