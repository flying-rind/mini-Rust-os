//! 内核中的文件抽象

use alloc::sync::Arc;
use downcast_rs::DowncastSync;

pub use inode::{OSInode, OpenFlags, ROOT_INODE, init, open_file};
pub use pipe::*;
use rcore_fs::vfs::INode;
use rcore_fs::vfs::Result;
pub use stdio::*;
use user_syscall::SysResult;

/// OS看到的文件抽象，只关心字节流的读写
pub trait File: Sync + Send + DowncastSync {
    /// 是否可读
    fn readable(&self) -> bool;
    /// 是否可写
    fn writable(&self) -> bool;
    /// 读取文件到buf中，返回实际读取的字节数
    async fn read(&self, buf: &mut [u8]) -> SysResult;
    /// 从buf中写入文件，返回实际写入的字节数
    fn write(&self, buf: &[u8]) -> usize;
    /// Lookup path from current INode, and follow symlinks at most follow_times times
    fn lookup_follow(&self, path: &str, max_follow: usize) -> Result<Arc<dyn INode>>;
}

/// 内核使用的Inode类型
mod inode;
// / 管道抽象
mod pipe;
// /// 标准输入输出抽象
mod stdio;
