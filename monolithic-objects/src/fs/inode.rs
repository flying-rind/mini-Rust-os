//! OSInode
use alloc::sync::Arc;
use hal::SysError;
use rcore_fs::vfs::INode;
use spin::Mutex;
use user_syscall::SysResult;

/// OS里操作的索引节点类型，封装了easy-fs中的Inode
///
/// 内核以这个结构来操作一个磁盘文件
pub struct OSInode {
    /// 是否可读
    readable: bool,
    /// 是否可写
    writable: bool,
    /// 偏移
    offset: Mutex<usize>,
    /// 封装rcore-fs中的Inode
    inode: Mutex<Arc<dyn INode>>,
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Arc<dyn INode>) -> Self {
        Self {
            readable,
            writable,
            offset: Mutex::new(0),
            inode: Mutex::new(inode),
        }
    }

    /// Readable
    pub fn readable(&self) -> bool {
        self.readable
    }

    /// Writable
    pub fn writable(&self) -> bool {
        self.writable
    }

    /// Read file to buf.
    pub fn read(&self, buf: &mut [u8]) -> SysResult {
        if !self.readable() {
            return Err(SysError::EPERM);
        }
        let (mut offset, inode) = (self.offset.lock(), self.inode.lock());
        let n = inode.read_at(*offset, buf)?;
        *offset += n;
        Ok(n)
    }

    /// Write to file with data from buf.
    pub fn write(&self, buf: &[u8]) -> SysResult {
        if !self.writable() {
            return Err(SysError::EPERM);
        }
        let (mut offset, inode) = (self.offset.lock(), self.inode.lock());
        let n = inode.write_at(*offset, buf)?;
        *offset += n;
        Ok(n)
    }

    /// Lookup from myself.
    pub fn lookup_follow(
        &self,
        path: &str,
        max_follow: usize,
    ) -> rcore_fs::vfs::Result<Arc<dyn INode>> {
        self.inode.lock().lookup_follow(path, max_follow)
    }
}
