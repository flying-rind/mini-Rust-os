//! 定义内核使用的Inode结构，为其实现文件访问接口
use super::File;
use crate::drivers::BlockDriverWrapper;
use crate::println;
use crate::*;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use rcore_fs::dev::block_cache::BlockCache;
use rcore_fs::vfs::FileSystem;
use rcore_fs::vfs::FileType;
use rcore_fs::vfs::INode;
use rcore_fs_sfs::SimpleFileSystem;
use spin::Mutex;

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

/// OS里操作的索引节点类型，封装了easy-fs中的Inode
///
/// 内核以这个结构来操作一个磁盘文件
pub struct OSInode {
    /// 是否可读
    readable: bool,
    /// 是否可写
    writable: bool,
    /// 偏移
    offset: Mutex<usize>,
    /// 封装rcore-fs中的Inode
    inode: Mutex<Arc<dyn INode>>,
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Arc<dyn INode>) -> Self {
        Self {
            readable,
            writable,
            offset: Mutex::new(0),
            inode: Mutex::new(inode),
        }
    }

    /// 读取一个I结点索引的所有数据
    pub fn read_all(&self) -> Vec<u8> {
        let inode = self.inode.lock();
        let size = inode.metadata().unwrap().size;
        let mut buffer = vec![0u8; size];
        let _ = inode.read_at(0, buffer.as_mut_slice());
        buffer
    }
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }

    fn writable(&self) -> bool {
        self.writable
    }

    fn read(&self, buf: &mut [u8]) -> usize {
        let (mut offset, inode) = (self.offset.lock(), self.inode.lock());
        let n = inode.read_at(*offset, buf);
        let n = n.unwrap();
        *offset += n;
        n
    }

    fn write(&self, buf: &[u8]) -> usize {
        let (mut offset, inode) = (self.offset.lock(), self.inode.lock());
        let n = inode.write_at(*offset, buf);
        let n = n.unwrap();
        *offset += n;
        n
    }

    fn lookup_follow(
        &self,
        path: &str,
        max_follow: usize,
    ) -> rcore_fs::vfs::Result<Arc<dyn INode>> {
        self.inode.lock().lookup_follow(path, max_follow)
    }
}

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
    for app in ROOT_INODE.list().unwrap() {
        println!("{}", app);
    }
    println!("**************/");
}

/// 从全局ROOT_INODE打开文件
pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        match ROOT_INODE.find(name) {
            Ok(inode) => {
                let _ = inode.resize(0);
                Some(Arc::new(OSInode::new(readable, writable, inode)))
            }
            Err(_) => {
                // TODO: MODE如何设置？
                ROOT_INODE
                    .create(name, FileType::File, 0o666)
                    .map(|inode| Arc::new(OSInode::new(readable, writable, inode)))
                    .ok()
            }
        }
    } else {
        ROOT_INODE
            .find(name)
            .map(|inode| {
                if flags.contains(OpenFlags::TRUNC) {
                    let _ = inode.resize(0);
                }
                Arc::new(OSInode::new(readable, writable, inode))
            })
            .ok()
    }
}
