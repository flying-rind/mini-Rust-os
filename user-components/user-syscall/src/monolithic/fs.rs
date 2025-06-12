//! 文件系统类系统调用

use crate::{sys_write, SysResult};

use super::error::SysError;
use rcore_fs::vfs::FsError;

impl From<FsError> for SysError {
    fn from(value: FsError) -> Self {
        match value {
            FsError::NotSupported => SysError::ENOSYS,
            FsError::NotFile => SysError::EISDIR,
            FsError::IsDir => SysError::EISDIR,
            FsError::NotDir => SysError::ENOTDIR,
            FsError::EntryNotFound => SysError::ENOENT,
            FsError::EntryExist => SysError::EEXIST,
            FsError::NotSameFs => SysError::EXDEV,
            FsError::InvalidParam => SysError::EINVAL,
            FsError::NoDeviceSpace => SysError::ENOMEM,
            FsError::DirRemoved => SysError::ENOENT,
            FsError::DirNotEmpty => SysError::ENOTEMPTY,
            FsError::WrongFs => SysError::EINVAL,
            FsError::DeviceError => SysError::EIO,
            FsError::IOCTLError => SysError::EINVAL,
            FsError::NoDevice => SysError::EINVAL,
            FsError::Again => SysError::EAGAIN,
            FsError::SymLoop => SysError::ELOOP,
            FsError::Busy => SysError::EBUSY,
            FsError::Interrupted => SysError::EINTR,
        }
    }
}

/// Write to file descriptor
pub fn write(fd: usize, buf: &[u8]) -> SysResult {
    let buf_ptr = buf.as_ptr() as *const u8;
    let size = buf.len();
    sys_write(fd, buf_ptr, size)
}
