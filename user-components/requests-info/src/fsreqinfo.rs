//! 用户向内核线程发送的文件系统相关请求描述信息
use super::*;

pub type Fd = usize;
pub type Pid = usize;
pub type BufPtr = usize;
pub type BufLen = usize;
pub type PathPtr = usize;
pub type FLAGS = u32;
pub type FdPtr = usize;
pub type ResultPtr = usize;

/// 文件系统类请求描述信息
#[derive(Debug, Clone, Copy)]
pub enum FsReqDescription {
    /// 读磁盘文件，在sys_read中被构造
    Read(Pid, Fd, BufPtr, BufLen, ResultPtr),
    /// 写磁盘文件，在sys_write中被构造
    Write(Pid, Fd, BufPtr, BufLen, ResultPtr),
    /// 打开一个磁盘文件，将句柄写入FdPtr中，
    /// 在sys_open中构造
    Open(Pid, PathPtr, FLAGS, FdPtr),
}

impl CastBytes for FsReqDescription {}
