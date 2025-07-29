//! 文件类系统调用
//! fcntl和ioctl功能还不完善，后期根据需要添加

mod fcntl;
mod ioctl;

use crate::Syscall;
use alloc::vec;
use fcntl::{F_DUPFD_CLOEXEC, F_GETFD, F_GETFL, F_SETFD, F_SETFL};
use hal::user::UserOutPtr;
use log::info;
use monolithic_objects::fs::file::File;
use monolithic_objects::fs::iovec::{IoVec, IoVecs};
use user_syscall::SysResult;

impl Syscall<'_> {
    /// Write to a file descriptor
    pub fn sys_write(&mut self, fd: usize, buf: *const u8, size: usize) -> SysResult {
        // info!("write, fd = {}", fd);
        let mut proc = self.process();
        // FIXME: Should check first.
        let slice = unsafe { core::slice::from_raw_parts(buf, size) };
        let file = proc.get_file(fd)?;
        let len = file.write(slice)?;
        Ok(len as _)
    }

    /// The writev() system call writes iovcnt buffers of data described
    /// by iov to the file associated with the file descriptor fd ("gather
    /// output").
    pub fn sys_writev(&mut self, fd: usize, iovec: *const IoVec, iovcnt: usize) -> SysResult {
        info!("writev: fd: {}, iov: {:?}, cnt: {}", fd, iovec, iovcnt);
        let iovecs = unsafe { IoVecs::new(iovec, iovcnt)? };
        let buf = iovecs.read_all_to_vec();
        let mut proc = self.process();
        let file = proc.get_file(fd)?;
        let len = file.write(&buf)?;
        Ok(len)
    }

    /// Read from a file descriptor
    ///
    /// See [read(2)](https://man7.org/linux/man-pages/man2/read.2.html)
    pub async fn sys_read(&mut self, fd: usize, mut base: UserOutPtr<u8>, len: usize) -> SysResult {
        let mut proc = self.process();
        let file = proc.get_file(fd)?;
        let mut buf = vec![0u8; len];
        let len = file.read(&mut buf).await?;
        let _ = base.write_array(&buf);
        Ok(len as _)
    }

    /// Implementation of dup, dup2, dup3.
    ///
    /// [dup(2)](https://man7.org/linux/man-pages/man2/dup.2.html)
    pub fn dup_impl(&mut self, oldfd: usize, newfd: usize, flags: usize) -> SysResult {
        let mut proc = self.process();
        proc.files.remove(&newfd);

        let file = proc.get_file(oldfd)?.dup(flags != 0);
        proc.files.insert(newfd, file);
        Ok(newfd)
    }

    /// Dup2.
    pub fn sys_dup2(&mut self, oldfd: usize, newfd: usize) -> SysResult {
        info!("dup2: from {} to {}", oldfd, newfd);
        self.dup_impl(oldfd, newfd, 0)
    }

    /// Dup3.
    pub fn sys_dup3(&mut self, oldfd: usize, newfd: usize, flags: usize) -> SysResult {
        info!(
            "dup3: form {} to {} with flags = {:#x}",
            oldfd, newfd, flags
        );
        self.dup_impl(oldfd, newfd, flags)
    }

    /// fcntl() performs one of the operations described below on the open
    /// file descriptor fd.  The operation is determined by op.
    ///
    /// [fcntl(2)](https://man7.org/linux/man-pages/man2/fcntl.2.html)
    ///
    /// FIXME: Add more ops.
    pub fn sys_fcntl(&mut self, fd: usize, op: usize, arg: usize) -> SysResult {
        info!("fcntl: fd: {}, cmd: {:#x}, arg: {}", fd, op, arg);
        let mut proc = self.process();
        let file = proc.get_file(fd)?;
        match file {
            File::FileHandle(file) => match op {
                F_SETFD => {
                    file.fd_cloexec = (arg & 1) != 0;
                    Ok(0)
                }
                F_GETFD => Ok(file.fd_cloexec as usize),
                F_SETFL => {
                    file.set_options(arg);
                    Ok(0)
                }
                F_GETFL => {
                    self.unimplemented("F_GETFL");
                }
                F_DUPFD_CLOEXEC => {
                    info!("fcntl: dupfd_cloexec: arg: {:#x}", arg);
                    // let file_like = proc.get_file_like(fd1)?.clone();
                    let new_fd = proc.get_free_fd_from(arg);
                    core::mem::drop(proc);
                    self.dup_impl(fd, new_fd, 1)
                }
                _ => Ok(0),
            },
            _ => {
                unimplemented!("Not supported yet.")
            }
        }
    }

    /// The ioctl() system call manipulates the underlying device
    /// parameters of special files.  In particular, many operating
    /// characteristics of character special files (e.g., terminals) may
    /// be controlled with ioctl() operations.  The argument fd must be an
    /// open file descriptor.
    ///
    /// [ioctl(2)](https://man7.org/linux/man-pages/man2/ioctl.2.html)
    ///
    /// FIXME: Add more ops.
    pub fn sys_ioctl(
        &mut self,
        fd: usize,
        op: usize,
        arg1: usize,
        arg2: usize,
        arg3: usize,
    ) -> SysResult {
        info!(
            "ioctl: fd: {}, request: {:#x}, args: {:#x} {:#x} {:#x}",
            fd, op, arg1, arg2, arg3
        );
        let mut proc = self.process();
        let file = proc.get_file(fd)?;
        file.ioctl(op, arg1, arg2, arg3)
    }
}
