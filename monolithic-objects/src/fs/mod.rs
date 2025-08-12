//! 文件系统类对象
pub mod file;
pub mod filehandle;
pub mod iovec;
mod tty;

use crate::fs::tty::TTY;
use alloc::sync::Arc;
pub use filehandle::OpenFlags;
use hal::println;
use hybrid_objects::drivers::{BLK_DRIVERS, BlockDriverWrapper};
use lazy_static::lazy_static;
use rcore_fs::dev::block_cache::BlockCache;
use rcore_fs::vfs::FileType;
use rcore_fs::vfs::INode;
use rcore_fs_devfs::DevFS;
use rcore_fs_devfs::special::NullINode;
use rcore_fs_mountfs::MountFS;
use rcore_fs_sfs::SimpleFileSystem;

// 初始化文件系统根节点
lazy_static! {
    pub static ref ROOT_INODE: Arc<dyn INode> = {
        let device = {
            let driver = BlockDriverWrapper(BLK_DRIVERS.read().iter().next().cloned().unwrap());
            Arc::new(BlockCache::new(driver, 0x100))
        };
        let sfs = SimpleFileSystem::open(device).expect("failed to open SFS");
        let rootfs = MountFS::new(sfs);
        let root = rootfs.mountpoint_root_inode();

        // Create DevFs.
        let devfs = DevFS::new();
        devfs.root().add("null", Arc::new(NullINode::default())).expect("Failed to mknod /dev/null");
        devfs.root().add("tty", TTY.clone()).expect("failed to mknod /dev/tty");
        // mount DevFS at /dev
        let dev = root.find(true, "dev").unwrap_or_else(|_| {
            root.create("dev", FileType::Dir, 0o666).expect("failed to mkdir /dev")
        });
        dev.mount(devfs).expect("Failed to mount DevFS");
        root
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
