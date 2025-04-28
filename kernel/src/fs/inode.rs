//! 定义内核使用的Inode结构，为其实现文件访问接口
use super::File;
use crate::drivers::BlockDriverWrapper;
use crate::println;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use rcore_fs::dev::block_cache::BlockCache;
use rcore_fs::vfs::FileSystem;
use rcore_fs::vfs::FileType;
use rcore_fs::vfs::INode;
use rcore_fs_sfs::SimpleFileSystem;

// pub static ROOT_INODE: Cell<Arc<Inode>> = unsafe { transmute([1u8; size_of::<Arc<Inode>>()]) };

// 初始化文件系统根节点
lazy_static! {
    pub static ref ROOT_INODE: Arc<dyn INode> = {
        let device = {
            let driver = BlockDriverWrapper(
                crate::drivers::BLK_DRIVERS
                    .read()
                    .iter()
                    .next()
                    .expect("Block device not found")
                    .clone(),
            );
            Arc::new(BlockCache::new(driver, 0x100))
        };
        let sfs = SimpleFileSystem::open(device).expect("failed to open SFS");
        sfs.root_inode()
    };
}

/// 文件系统初始化,打印目录
pub fn init() {
    println!("/****APPS****/");
    for app in ROOT_INODE.list() {
        println!("{:?}", app);
    }
    println!("**************/");
}

bitflags::bitflags! {
    /// 打开文件时的读写权限
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        /// 创建时清空
        const TRUNC = 1 << 10;
    }
}

impl OpenFlags {
    /// 获取读写权限
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

/// 从全局ROOT_INODE打开文件
pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<dyn INode>> {
    // let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        match ROOT_INODE.find(name) {
            Ok(inode) => {
                inode.resize(0);
                Some(inode)
            }
            Err(_) => {
                // TODO: MODE如何设置？
                ROOT_INODE.create(name, FileType::File, 0o666).ok()
            }
        }
    } else {
        ROOT_INODE
            .find(name)
            .map(|inode| {
                if flags.contains(OpenFlags::TRUNC) {
                    inode.resize(0);
                }
                inode
            })
            .ok()
    }
}
