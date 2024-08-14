//! 文件相关的用户系统调用封装
use super::*;

bitflags::bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

/// 当前进程打开一个文件
///
/// 成功返回fd，否则返回None
pub fn open(path: &str, flags: OpenFlags) -> Option<usize> {
    let path_ptr = &path as *const &str as usize;
    let (fd, _) = sys_open(path_ptr, flags.bits as _);
    if fd == usize::MAX {
        return None;
    }
    return Some(fd);
}

/// 读取当前进程的一个文件
///
/// 成功返回读取的字节数，否则返回None
pub fn read(fd: usize, buf: &mut [u8]) -> Option<usize> {
    let buf_ptr = buf.as_mut_ptr() as usize;
    let (read_bytes, _) = sys_read(fd, buf_ptr, buf.len());
    // println!("read_bytes: {:x}", read_bytes);
    if read_bytes == usize::MAX {
        return None;
    }
    return Some(read_bytes);
}

/// 写入当前进程的一个文件
///
/// 成功返回写入的字节数，否则返回None
pub fn write(fd: usize, buf: &[u8]) -> Option<usize> {
    let buf_ptr = buf.as_ptr() as usize;
    let (write_bytes, _) = sys_write(fd, buf_ptr, buf.len());
    if write_bytes == usize::MAX {
        return None;
    }
    return Some(write_bytes);
}

/// 关闭当前进程的一个文件
///
/// 成功返回0，否则返回None
pub fn close(fd: usize) -> Option<usize> {
    let (ret1, _) = sys_close(fd);
    if ret1 == usize::MAX {
        return None;
    }
    Some(ret1)
}

/// 创建管道，返回读端和写端的fd
pub fn make_pipe() -> (usize, usize) {
    sys_pipe()
}

/// 复制当前进程的一个文件
///
/// 若成功返回复制后的fd，否则返回None
pub fn dup(fd: usize) -> Option<usize> {
    let (ret1, _) = sys_dup(fd);
    if ret1 == usize::MAX {
        return None;
    }
    Some(ret1)
}

/// 列出可用用户app
pub fn ls() -> (usize, usize) {
    sys_ls()
}
