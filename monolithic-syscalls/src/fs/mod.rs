//! 文件类系统调用
//! fcntl和ioctl功能还不完善，后期根据需要添加

mod fcntl;
mod ioctl;
mod stat;

use crate::Syscall;
use alloc::vec;
use fcntl::{F_DUPFD_CLOEXEC, F_GETFD, F_GETFL, F_SETFD, F_SETFL};
use hal::SysError;
use hal::check_n_clone_cstr;
use hal::user::UserOutPtr;
use log::error;
use log::info;
use monolithic_objects::fs::OpenFlags;
use monolithic_objects::fs::file::File;
use monolithic_objects::fs::filehandle::FileHandle;
use monolithic_objects::fs::iovec::{IoVec, IoVecs};
use rcore_fs::vfs::FsError;
pub use stat::*;
use user_syscall::SysResult;

/// Split a `path` str to `(base_path, file_name)`
fn split_path(path: &str) -> (&str, &str) {
    let mut split = path.trim_end_matches('/').rsplitn(2, '/');
    let file_name = split.next().unwrap();
    let mut dir_path = split.next().unwrap_or(".");
    if dir_path == "" {
        dir_path = "/";
    }
    (dir_path, file_name)
}

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
        info!("writev: fd: {}, iov_ptr: {:?}, cnt: {}", fd, iovec, iovcnt);
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
                F_GETFL => self.unimplemented("F_GETFL", Ok(0)),
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

    /// The open() system call opens the file specified by pathname.  If
    /// the specified file does not exist, it may optionally (if O_CREAT
    /// is specified in flags) be created by open().
    ///
    /// [open(2)](https://man7.org/linux/man-pages/man2/open.2.html)
    pub fn sys_open(&mut self, path: *const u8, flags: usize, mode: usize) -> SysResult {
        const AT_FDCWD: usize = -100isize as usize;
        self.sys_openat(AT_FDCWD, path, flags, mode)
    }

    /// Open and possibly create a file.
    pub fn sys_openat(
        &mut self,
        dir_fd: usize,
        path: *const u8,
        flags: usize,
        mode: usize,
    ) -> SysResult {
        let mut proc = self.process();
        let path = check_n_clone_cstr(path)?;
        let flags = OpenFlags::from_bits_truncate(flags);
        info!(
            "openat: dir_fd: {}, path: {:?}, flags: {:#?}, mode: {:#o}",
            dir_fd as isize, path, flags, mode
        );
        let inode = if flags.contains(OpenFlags::CREATE) {
            let (dir_path, file_name) = split_path(&path);
            let dir_inode = proc.lookup_inode_at(dir_fd, dir_path, true)?;
            match dir_inode.find(file_name) {
                Ok(file_inode) => {
                    if flags.contains(OpenFlags::EXCLUSIVE) {
                        return Err(SysError::EEXIST);
                    }
                    if flags.contains(OpenFlags::TRUNCATE) {
                        if let Err(_e) = file_inode.resize(0) {
                            error!("Resize error!");
                        }
                    }
                    file_inode
                }
                Err(FsError::EntryNotFound) => {
                    let inode =
                        dir_inode.create(file_name, rcore_fs::vfs::FileType::File, mode as _)?;
                    inode
                }
                Err(e) => return Err(SysError::from(e)),
            }
        } else {
            proc.lookup_inode_at(dir_fd, &path, true)?
        };
        let file = File::FileHandle(FileHandle::new(
            inode,
            flags.contains(OpenFlags::CLOEXEC),
            flags.to_options(),
            false,
        ));
        Ok(proc.add_file(file))
    }

    /// close() closes a file descriptor, so that it no longer refers to
    /// any file and may be reused.  Any record locks (see fcntl(2)) held
    /// on the file it was associated with, and owned by the process, are
    /// removed regardless of the file descriptor that was used to obtain
    /// the lock.  This has some unfortunate consequences and one should
    /// be extra careful when using advisory record locking.  See fcntl(2)
    /// for discussion of the risks and consequences as well as for the
    /// (probably preferred) open file description locks.
    ///
    /// [close(2)](https://man7.org/linux/man-pages/man2/close.2.html)
    pub fn sys_close(&mut self, fd: usize) -> SysResult {
        info!("close: fd: {:?}", fd);
        let mut proc = self.process();

        proc.files.remove(&fd).ok_or(SysError::EBADF)?;
        Ok(0)
    }

    /// These functions return information about a file, in the buffer
    /// pointed to by statbuf.  No permissions are required on the file
    /// itself, but—in the case of stat(), fstatat(), and lstat()—execute
    /// (search) permission is required on all of the directories in
    /// pathname that lead to the file.
    ///
    /// fstat() is identical to stat(), except that the file about which
    /// information is to be retrieved is specified by the file descriptor
    /// fd.
    ///
    /// [fstat(2)](https://man7.org/linux/man-pages/man2/stat.2.html)
    pub fn sys_fstatat(
        &mut self,
        dirfd: usize,
        path: *const u8,
        stat_ptr: *mut Stat,
        flags: usize,
    ) -> SysResult {
        let proc = self.process();
        let path = check_n_clone_cstr(path)?;
        let flags = AtFlags::from_bits_truncate(flags);
        info!(
            "fstatat: dirfd: {}, path: {:?}, stat_ptr: {:?}, flags: {:?}",
            dirfd as isize, path, stat_ptr, flags
        );
        let inode =
            proc.lookup_inode_at(dirfd, &path, !flags.contains(AtFlags::SYMLINK_NOFOLLOW))?;
        let stat_ref: &'static mut Stat = unsafe {
            let slice = core::slice::from_raw_parts_mut::<'static>(stat_ptr, 1);
            &mut slice[0]
        };
        let stat = Stat::from(inode.metadata()?);
        *stat_ref = stat;
        Ok(0)
    }

    /// See [`Self::sys_fstatat`]
    pub fn sys_stat(&mut self, path: *const u8, stat_ptr: *mut Stat) -> SysResult {
        info!("stat: path: {:?}, stat_ptr: {:?}", path, stat_ptr);
        /// Pathname is interpreted relative to the current working directory(CWD)
        const AT_FDCWD: usize = -100isize as usize;
        self.sys_fstatat(AT_FDCWD, path, stat_ptr, 0)
    }

    /// Get current working dir.
    pub fn sys_getcwd(&mut self, buf: *mut u8, len: usize) -> SysResult {
        let proc = self.process();
        info!("getcwd: buf: {:?}, len: {:#x}", buf, len);
        let buf: &'static mut [u8] = unsafe { core::slice::from_raw_parts_mut(buf, len) };
        if proc.cwd.len() + 1 > len {
            return Err(SysError::ERANGE);
        }
        unsafe {
            hal::write_cstr(buf.as_mut_ptr(), &proc.cwd);
        }
        Ok(buf.as_ptr() as usize)
    }
}
