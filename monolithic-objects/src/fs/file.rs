//! File Abstract.
//! 文件抽象

use crate::fs::filehandle::FileHandle;
use alloc::sync::Arc;
use hybrid_objects::fs::{Stdin, Stdout};
use rcore_fs::vfs::{FsError, INode};
use user_syscall::SysResult;

/// File
#[derive(Clone)]
pub enum File {
    FileHandle(FileHandle),
    Stdin(Stdin),
    Stdout(Stdout),
}

impl File {
    /// Dup with fd_cloexec specified.
    pub fn dup(&self, fd_cloexec: bool) -> File {
        use File::*;
        match self {
            FileHandle(file) => FileHandle(file.dup(fd_cloexec)),
            _ => unimplemented!(),
        }
    }

    /// Read the file. Maybe async.
    pub async fn read(&self, buf: &mut [u8]) -> SysResult {
        use File::*;
        match self {
            FileHandle(filehandle) => filehandle.read(buf),
            Stdin(stdin) => stdin.read(buf),
            Stdout(stdout) => stdout.read(buf),
        }
    }

    /// Write to the file. Maybe async.
    pub fn write(&self, buf: &[u8]) -> SysResult {
        use File::*;
        match self {
            FileHandle(filehandle) => filehandle.write(buf),
            Stdin(stdin) => stdin.write(buf),
            Stdout(stdout) => stdout.write(buf),
        }
    }

    /// Lookup from the file.
    pub fn lookup_follow(&self, path: &str, max_follow: usize) -> Result<Arc<dyn INode>, FsError> {
        use File::*;
        match self {
            FileHandle(filehandle) => filehandle.lookup_follow(path, max_follow),
            Stdin(stdin) => stdin.lookup_follow(path, max_follow),
            Stdout(stdout) => stdout.lookup_follow(path, max_follow),
        }
    }
}
