//! 文件相关系统调用

use super::{uaccess::*, *};
use crate::{fs::*, *};

/// 将当前进程中第一个可用的文件描述符指向原表中fd指向的文件
///
/// 若成功，返回使用的fd，否则返回-1
///
/// 与close结合使用
pub fn sys_dup(fd: usize) -> isize {
    let t = task::current();
    let file = if let Some(Some(x)) = t.proc.files.get(fd) {
        x
    } else {
        return -1;
    };
    let file = file.clone();
    t.proc.add_file(file) as _
}

/// 打开根节点下的一个磁盘文件,并加入当前进程文件表
///
/// 若成功返回fd，否则返回-1
pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let t = task::current();
    let path = try_!(read_cstr(path), EFAULT);
    if let Some(inode) = open_file(&path, OpenFlags::from_bits(flags).unwrap()) {
        t.proc.add_file(inode) as _
    } else {
        -1
    }
}

/// 从当前进程的文件表中移除描述符为fd的文件
pub fn sys_close(fd: usize) -> isize {
    let t = task::current();
    if let Some(x) = t.proc.files.get_mut(fd) {
        if core::mem::replace(x, None).is_some() {
            return 0;
        }
    }
    -1
}

/// 创建管道
///
/// 为当前进程添加读端写端两个文件
///
/// 将读端和写端两个文件的描述符fd写入用户态pipe内存中
pub fn sys_pipe(pipe: *mut usize) -> isize {
    let t = task::current();
    let (r, w) = make_pipe();
    let (r, w) = (t.proc.add_file(r), t.proc.add_file(w));
    try_!(pipe.write_user(r), EFAULT);
    try_!(unsafe { pipe.add(1) }.write_user(w), EFAULT);
    0
}

/// 使用当前进程文件表中描述符为fd的文件write
pub fn sys_write(fd: usize, ptr: *const u8, len: usize) -> isize {
    let t = task::current();
    let root_pa = t.proc.root_pa();
    let file = if let Some(Some(x)) = t.proc.files.get(fd) {
        x
    } else {
        return -1;
    };
    if !file.writable() {
        return -1;
    }
    let buf = try_!(validate_buf(root_pa, ptr, len, false), EFAULT);
    file.write(buf) as _
}

/// 使用当前进程文件表中描述符为fd的文件read
pub fn sys_read(fd: usize, ptr: *mut u8, len: usize) -> isize {
    let t = task::current();
    let root_pa = t.proc.root_pa();
    let file = if let Some(Some(x)) = &t.proc.files.get(fd) {
        x
    } else {
        return -1;
    };
    if !file.readable() {
        return -1;
    }
    let buf = try_!(validate_buf(root_pa, ptr, len, true), EFAULT);
    file.read(buf) as _
}
