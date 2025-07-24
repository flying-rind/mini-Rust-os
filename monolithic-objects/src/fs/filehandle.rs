//! FileHandle
use alloc::sync::Arc;
use rcore_fs::vfs::INode;
use spin::RwLock;
use user_syscall::SysResult;

use crate::fs::O_NONBLOCK;

/// Open file descriptions.
pub struct OpenFileDescription {
    options: OpenOptions,
    offset: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct OpenOptions {
    pub read: bool,
    pub write: bool,
    /// Before each write, the file offset is positioned at the end of the file.
    pub append: bool,
    pub nonblock: bool,
}

/// Kernel manipulates an Inode with this struct.
#[derive(Clone)]
pub struct FileHandle {
    /// 封装rcore-fs中的Inode
    inode: Arc<dyn INode>,
    /// fd_closexec
    pub fd_cloexec: bool,
    /// Openoption.
    description: Arc<RwLock<OpenFileDescription>>,
    /// If is pipe.
    pub pipe: bool,
}

impl FileHandle {
    /// Create a new filehandle.
    pub fn new(inode: Arc<dyn INode>, fd_cloexec: bool, options: OpenOptions, pipe: bool) -> Self {
        Self {
            inode,
            fd_cloexec,
            description: Arc::new(RwLock::new(OpenFileDescription {
                options: options,
                offset: 0,
            })),
            pipe,
        }
    }

    /// Do almost as default clone does, but with fd_cloexec specified.
    pub fn dup(&self, fd_cloexec: bool) -> Self {
        FileHandle {
            inode: self.inode.clone(),
            description: self.description.clone(),
            fd_cloexec, // this field do not share
            pipe: self.pipe,
        }
    }

    /// Read file to buf.
    pub fn read(&self, buf: &mut [u8]) -> SysResult {
        let (offset, inode) = (self.description.read().offset, self.inode.clone());
        let n = inode.read_at(offset, buf)?;
        self.description.write().offset += n;
        Ok(n)
    }

    /// Write to file with data from buf.
    pub fn write(&self, buf: &[u8]) -> SysResult {
        let (offset, inode) = (self.description.read().offset, self.inode.clone());
        let n = inode.write_at(offset, buf)?;
        self.description.write().offset += n;
        Ok(n)
    }

    /// Lookup from myself.
    pub fn lookup_follow(
        &self,
        path: &str,
        max_follow: usize,
    ) -> rcore_fs::vfs::Result<Arc<dyn INode>> {
        self.inode.lookup_follow(path, max_follow)
    }

    /// Set open options.
    pub fn set_options(&self, arg: usize) {
        let mut options = self.description.write().options;
        options.nonblock = (arg & O_NONBLOCK) != 0;
    }
}
