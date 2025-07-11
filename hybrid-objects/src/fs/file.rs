//! 文件抽象

use crate::fs::{OSInode, Pipe, Stdin, Stdout};
use alloc::sync::Arc;
use rcore_fs::vfs::{FsError, INode};
use user_syscall::SysResult;

/// File
pub enum File {
    OSInode(OSInode),
    Pipe(Pipe),
    Stdin(Stdin),
    Stdout(Stdout),
}

impl File {
    /// If can read.
    pub fn readable(&self) -> bool {
        use File::*;
        match self {
            OSInode(osinode) => osinode.readable(),
            Pipe(pipe) => pipe.readable(),
            Stdin(_stdin) => false,
            Stdout(_stdout) => true,
        }
    }

    /// If can write.
    pub fn writable(&self) -> bool {
        use File::*;
        match self {
            OSInode(osinode) => osinode.writable(),
            Pipe(pipe) => pipe.writable(),
            Stdin(_stdin) => true,
            Stdout(_stdout) => false,
        }
    }

    /// Read the file. Maybe async.
    pub async fn read(&self, buf: &mut [u8], fd: usize) -> SysResult {
        use File::*;
        match self {
            OSInode(osinode) => osinode.send_read_req(buf, fd).await,
            Pipe(pipe) => pipe.read(buf).await,
            Stdin(stdin) => stdin.read(buf),
            Stdout(stdout) => stdout.read(buf),
        }
    }

    /// Write to the file. Maybe async.
    pub async fn write(&self, buf: &[u8], fd: usize) -> SysResult {
        use File::*;
        match self {
            OSInode(osinode) => osinode.send_write_req(buf, fd).await,
            Pipe(pipe) => pipe.write(buf),
            Stdin(stdin) => stdin.write(buf),
            Stdout(stdout) => stdout.write(buf),
        }
    }

    /// Lookup from the file.
    pub fn lookup_follow(&self, path: &str, max_follow: usize) -> Result<Arc<dyn INode>, FsError> {
        use File::*;
        match self {
            OSInode(osinode) => osinode.lookup_follow(path, max_follow),
            Pipe(pipe) => pipe.lookup_follow(path, max_follow),
            Stdin(stdin) => stdin.lookup_follow(path, max_follow),
            Stdout(stdout) => stdout.lookup_follow(path, max_follow),
        }
    }
}
