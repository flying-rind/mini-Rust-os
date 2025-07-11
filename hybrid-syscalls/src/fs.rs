//! 文件相关系统调用

use alloc::sync::Arc;
use fs::OSInode;
use fs::*;
use hal::SysError;
use hal::check_n_clone_cstr;
use hybrid_objects::current_proc;
use hybrid_objects::fs;
use log::error;
use log::info;
use rcore_fs::vfs::FsError;
use user_syscall::SysResult;

use crate::Syscall;

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
    /// Open and possibly create a file at the cwd.
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
        let proc = current_proc();
        let path = check_n_clone_cstr(path)?;
        let flags = OpenFlags::from_bits_truncate(flags);
        info!(
            "openat: dir_fd: {}, path: {:?}, flags: {:?}, mode: {:#o}",
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
        let (readable, writable) = flags.read_write();
        let file = Arc::new(File::OSInode(OSInode::new(readable, writable, inode)));
        Ok(proc.add_file(file))
    }

    /// 读取当前进程的fd对应的文件
    pub async fn sys_read(&mut self, fd: usize, buf_ptr: usize, buf_len: usize) -> SysResult {
        let current_proc = self.process();
        let file = current_proc.file_table().get(&fd).ok_or(SysError::ENOENT)?;
        let buf_ptr = buf_ptr as *mut u8;
        let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, buf_len) };
        let read_size = file.read(buf, fd).await?;
        Ok(read_size)
    }

    /// 写入当前进程的fd对应的文件
    pub async fn sys_write(&mut self, fd: usize, buf_ptr: usize, buf_len: usize) -> SysResult {
        let proc = self.process();
        let file_table = proc.file_table();
        let file = file_table.get(&fd).ok_or(SysError::ENOENT)?;
        let buf_ptr = buf_ptr as *const u8;
        let buf = unsafe { core::slice::from_raw_parts(buf_ptr, buf_len) };
        let write_size = file.write(buf, fd).await?;
        Ok(write_size)
    }

    /// 当前进程关闭描述符为fd的文件
    ///
    /// 成功返回0，否则返回usize::MAX
    pub fn sys_close(&mut self, fd: usize) -> SysResult {
        let current_proc = current_proc();
        let file_table = current_proc.file_table();
        file_table.remove(&fd).ok_or(SysError::EBADF)?;
        Ok(0)
    }

    /// 创建管道，返回读端和写端的fd
    pub fn sys_pipe(&mut self, fds: *mut u32) -> SysResult {
        let current_proc = current_proc();
        let fds = unsafe { core::slice::from_raw_parts_mut(fds, 2) };
        let (read_end, write_end) = make_pipe();
        let (read_fd, write_fd) = (
            current_proc.add_file(read_end),
            current_proc.add_file(write_end),
        );
        fds[0] = read_fd as _;
        fds[1] = write_fd as _;
        Ok(0)
    }

    // /// 复制一份文件，一般与close一起使用
    // ///
    // /// 若文件不存在则返回usize::MAX
    // pub fn sys_dup(fd: usize) -> (usize, usize) {
    //     let current_proc = current_proc();
    //     let file_table = current_proc.file_table();
    //     let file = if let Some(Some(f)) = file_table.get(fd) {
    //         f.clone()
    //     } else {
    //         return (usize::MAX, 0);
    //     };
    //     (current_proc.add_file(file), 0)
    // }

    // /// 列出可用用户app
    // pub fn sys_ls() -> SysResult {
    //     let step = 7;
    //     let apps = ROOT_INODE.list().unwrap();
    //     for i in (0..apps.len()).step_by(step) {
    //         for j in i..i + step {
    //             if j < apps.len() {
    //                 print!("{:<20}", apps[j]);
    //             } else {
    //                 break;
    //             }
    //         }
    //         println!("");
    //     }
    //     Ok(0)
    // }
}
