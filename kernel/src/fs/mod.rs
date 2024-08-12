//! 内核中的文件抽象

use downcast_rs::impl_downcast;
use downcast_rs::DowncastSync;

pub use inode::{init, open_file, OSInode, OpenFlags, ROOT_INODE};
pub use pipe::*;
pub use stdio::*;

/// OS看到的文件抽象，只关心字节流的读写
pub trait File: Sync + Send + DowncastSync {
    /// 是否可读
    fn readable(&self) -> bool;
    /// 是否可写
    fn writable(&self) -> bool;
    /// 读取文件到buf中，返回实际读取的字节数
    fn read(&self, buf: &mut [u8]) -> usize;
    /// 从buf中写入文件，返回实际写入的字节数
    fn write(&self, buf: &[u8]) -> usize;
}
impl_downcast!(sync File);

/// 内核使用的Inode类型
mod inode;
// / 管道抽象
mod pipe;
// /// 标准输入输出抽象
mod stdio;
