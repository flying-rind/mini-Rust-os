//! 文件系统类对象
pub mod file;
pub mod filehandle;
pub mod iovec;

use alloc::sync::Arc;
pub use filehandle::OpenFlags;
use hal::println;
use hybrid_objects::drivers::{BLK_DRIVERS, BlockDriverWrapper};
use lazy_static::lazy_static;
use rcore_fs::dev::block_cache::BlockCache;
use rcore_fs::vfs::FileSystem;
use rcore_fs::vfs::INode;
use rcore_fs_sfs::SimpleFileSystem;

// 初始化文件系统根节点
lazy_static! {
    pub static ref ROOT_INODE: Arc<dyn INode> = {
        let device = {
            let driver = BlockDriverWrapper(BLK_DRIVERS.read().iter().next().cloned().unwrap());
            Arc::new(BlockCache::new(driver, 0x100))
        };
        let sfs = SimpleFileSystem::open(device).expect("failed to open SFS");
        sfs.root_inode()
    };
}

/// 文件系统初始化,打印目录
#[allow(unused)]
pub fn init() {
    println!("/****APPS****/");
    for app in ROOT_INODE.list().unwrap() {
        println!("{}", app);
    }
    println!("**************/");
}
