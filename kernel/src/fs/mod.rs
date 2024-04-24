pub use inode::{init, open_file, OSInode, OpenFlags};
pub use stdio::*;
/// OS看到的文件抽象，只关心字节流的读写
pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: &mut [u8]) -> usize;
    fn write(&self, buf: &[u8]) -> usize;
}

/// 内核使用的Inode类型
mod inode;
/// 标准输入输出抽象
mod stdio;
