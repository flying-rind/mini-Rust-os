//! 定义内核使用的Inode结构，为其实现文件访问接口
use super::File;
use crate::{drivers::BLOCK_DEVICE, *};

use alloc::sync::Arc;
use easy_fs::{EasyFileSystem, Inode};

/// OS里操作的索引节点类型，封装了easy-fs中的Inode
///
/// 内核以这个结构来操作一个磁盘文件
pub struct OSInode {
    /// 是否可读
    readable: bool,
    /// 是否可写
    writable: bool,
    /// 偏移
    offset: Cell<usize>,
    /// 封装easy-fs中的Inode
    inode: Cell<Arc<Inode>>,
}

/// 全局变量：根节点的索引节点
pub static ROOT_INODE: Cell<Arc<Inode>> = unsafe { transmute([1u8; size_of::<Arc<Inode>>()]) };

/// 文件系统初始化,创建root inode
pub fn init() {
    let efs = EasyFileSystem::open(BLOCK_DEVICE.clone());
    unsafe {
        (ROOT_INODE.get_mut() as *mut Arc<Inode>).write(Arc::new(EasyFileSystem::root_inode(&efs)));
    }
    println!("/****APPS****/");
    for app in ROOT_INODE.ls() {
        println!("{}", app);
    }
    println!("**************/");
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Arc<Inode>) -> Self {
        Self {
            readable,
            writable,
            offset: Cell::new(0),
            inode: Cell::new(inode),
        }
    }

    /// 读取一个I结点索引的所有数据
    pub fn read_all(&self) -> Vec<u8> {
        let (offset, inode) = (self.offset.get_mut(), self.inode.get_mut());
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inode.read_at(*offset, &mut buffer);
            if len == 0 {
                break;
            }
            *offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }
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

/// 根据OpenFlags打开根节点下的文件
pub fn open_file(name: &str, flags: OpenFlags) -> Option<Rc<OSInode>> {
    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        if let Some(inode) = ROOT_INODE.find(name) {
            inode.clear();
            Some(Rc::new(OSInode::new(readable, writable, inode)))
        } else {
            // create file
            ROOT_INODE
                .create(name)
                .map(|inode| Rc::new(OSInode::new(readable, writable, inode)))
        }
    } else {
        ROOT_INODE.find(name).map(|inode| {
            if flags.contains(OpenFlags::TRUNC) {
                inode.clear();
            }
            Rc::new(OSInode::new(readable, writable, inode))
        })
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
        let (offset, inode) = (self.offset.get_mut(), self.inode.get_mut());
        let n = inode.read_at(*offset, buf);
        *offset += n;
        n
    }

    fn write(&self, buf: &[u8]) -> usize {
        let (offset, inode) = (self.offset.get_mut(), self.inode.get_mut());
        let n = inode.write_at(*offset, buf);
        *offset += n;
        n
    }
}
