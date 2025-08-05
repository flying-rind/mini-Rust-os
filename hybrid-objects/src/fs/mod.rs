//! 内核中的文件抽象

pub use blkwrap::*;
pub use file::*;
pub use inode::{OSInode, OpenFlags, ROOT_INODE};
pub use pipe::*;
pub use stdio::*;

mod blkwrap;
/// 文件
mod file;
/// 内核使用的Inode类型
mod inode;
/// 管道抽象
mod pipe;
/// 标准输入输出抽象
mod stdio;
