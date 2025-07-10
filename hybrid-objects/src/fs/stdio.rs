//! 定义标准输入输出，为其实现文件访问接口
use hal::{SysError, console::serial_receive};
use rcore_fs::vfs::FsError;
use user_syscall::SysResult;

use crate::*;

/// 标准输入
pub struct Stdin;
/// 标准输出
pub struct Stdout;

impl Stdin {
    pub fn readable(&self) -> bool {
        true
    }

    pub fn writable(&self) -> bool {
        false
    }

    /// 从串口读取一个字符到buf中
    pub fn read(&self, buf: &mut [u8]) -> SysResult {
        assert_eq!(buf.len(), 1);
        let c = serial_receive();
        buf[0] = c as _;
        Ok(1)
    }

    #[allow(unused)]
    pub fn write(&self, buf: &[u8]) -> SysResult {
        panic!("Cannot write to stdin!");
    }

    pub fn lookup_follow(
        &self,
        _path: &str,
        _max_follow: usize,
    ) -> rcore_fs::vfs::Result<alloc::sync::Arc<dyn rcore_fs::vfs::INode>> {
        Err(FsError::NotFile)
    }
}

impl Stdout {
    pub fn readable(&self) -> bool {
        false
    }

    pub fn writable(&self) -> bool {
        true
    }

    #[allow(unused)]
    pub fn read(&self, buf: &mut [u8]) -> SysResult {
        panic!("Cannot read from stdout!")
    }

    // 打印到串口（输出到主机屏幕）
    pub fn write(&self, buf: &[u8]) -> SysResult {
        if let Ok(str) = core::str::from_utf8(buf) {
            print!("{}", str);
            Ok(buf.len())
        } else {
            Err(SysError::EINVAL)
        }
    }

    pub fn lookup_follow(
        &self,
        _path: &str,
        _max_follow: usize,
    ) -> rcore_fs::vfs::Result<alloc::sync::Arc<dyn rcore_fs::vfs::INode>> {
        Err(rcore_fs::vfs::FsError::NotFile)
    }
}
