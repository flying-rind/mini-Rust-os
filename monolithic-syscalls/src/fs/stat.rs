//! Struct for sys_stat

use bitflags::bitflags;
use monolithic_objects::sync::TimeSpec;
use rcore_fs::vfs::{FileType, Metadata};

#[repr(C)]
#[derive(Debug)]
/// Copied form rcore.
///
/// [Stat structs](https://man7.org/linux/man-pages/man3/stat.3type.html)
pub struct Stat {
    /// ID of device containing file
    dev: u64,
    /// inode number
    ino: u64,
    /// number of hard links
    nlink: u64,

    /// file type and mode
    mode: StatMode,
    /// user ID of owner
    uid: u32,
    /// group ID of owner
    gid: u32,
    /// padding
    _pad0: u32,
    /// device ID (if special file)
    rdev: u64,
    /// total size, in bytes
    size: u64,
    /// blocksize for filesystem I/O
    blksize: u64,
    /// number of 512B blocks allocated
    blocks: u64,

    /// last access time
    atime: TimeSpec,
    /// last modification time
    mtime: TimeSpec,
    /// last status change time
    ctime: TimeSpec,
}

impl From<Metadata> for Stat {
    fn from(value: Metadata) -> Self {
        Stat {
            dev: value.dev as u64,
            ino: value.inode as u64,
            mode: StatMode::from_type_mode(value.type_, value.mode as u16),
            nlink: value.nlinks as u64,
            uid: value.uid as u32,
            gid: value.gid as u32,
            rdev: value.rdev as u64,
            size: value.size as u64,
            blksize: value.blk_size as u64,
            blocks: value.blocks as u64,
            atime: TimeSpec {
                sec: value.atime.sec as _,
                nsec: value.atime.nsec as _,
            },
            mtime: TimeSpec {
                sec: value.mtime.sec as _,
                nsec: value.mtime.nsec as _,
            },
            ctime: TimeSpec {
                sec: value.ctime.sec as _,
                nsec: value.ctime.nsec as _,
            },
            _pad0: 0,
        }
    }
}

bitflags! {
    #[derive(Debug)]
    pub struct StatMode: u32 {
        const NULL  = 0;
        /// Type
        const TYPE_MASK = 0o170000;
        /// FIFO
        const FIFO  = 0o010000;
        /// character device
        const CHAR  = 0o020000;
        /// directory
        const DIR   = 0o040000;
        /// block device
        const BLOCK = 0o060000;
        /// ordinary regular file
        const FILE  = 0o100000;
        /// symbolic link
        const LINK  = 0o120000;
        /// socket
        const SOCKET = 0o140000;

        /// Set-user-ID on execution.
        const SET_UID = 0o4000;
        /// Set-group-ID on execution.
        const SET_GID = 0o2000;

        /// Read, write, execute/search by owner.
        const OWNER_MASK = 0o700;
        /// Read permission, owner.
        const OWNER_READ = 0o400;
        /// Write permission, owner.
        const OWNER_WRITE = 0o200;
        /// Execute/search permission, owner.
        const OWNER_EXEC = 0o100;

        /// Read, write, execute/search by group.
        const GROUP_MASK = 0o70;
        /// Read permission, group.
        const GROUP_READ = 0o40;
        /// Write permission, group.
        const GROUP_WRITE = 0o20;
        /// Execute/search permission, group.
        const GROUP_EXEC = 0o10;

        /// Read, write, execute/search by others.
        const OTHER_MASK = 0o7;
        /// Read permission, others.
        const OTHER_READ = 0o4;
        /// Write permission, others.
        const OTHER_WRITE = 0o2;
        /// Execute/search permission, others.
        const OTHER_EXEC = 0o1;
    }
}

impl StatMode {
    fn from_type_mode(type_: FileType, mode: u16) -> Self {
        let type_ = match type_ {
            FileType::File => StatMode::FILE,
            FileType::Dir => StatMode::DIR,
            FileType::SymLink => StatMode::LINK,
            FileType::CharDevice => StatMode::CHAR,
            FileType::BlockDevice => StatMode::BLOCK,
            FileType::Socket => StatMode::SOCKET,
            FileType::NamedPipe => StatMode::FIFO,
        };
        let mode = StatMode::from_bits_truncate(mode as u32);
        type_ | mode
    }
}

bitflags! {
    /// Used by fstat.
    #[derive(Debug)]
    pub struct AtFlags: usize {
        const EMPTY_PATH = 0x1000;
        const SYMLINK_NOFOLLOW = 0x100;
    }
}
