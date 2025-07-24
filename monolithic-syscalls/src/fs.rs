//! 文件类系统调用

use crate::Syscall;
use alloc::vec;
use hal::user::UserOutPtr;
use log::info;
use monolithic_objects::fs::{F_DUPFD_CLOEXEC, F_GETFD, F_GETFL, F_SETFD, F_SETFL, file::File};
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
        let len = file.write(slice)?;
        Ok(len as _)
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
}
