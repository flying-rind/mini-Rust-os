//! 定义内核使用的Inode结构，为其实现文件访问接口
use crate::drivers::BlockDriverWrapper;
use crate::future::futures::WaitForKthread;
use crate::println;
use crate::*;
use alloc::sync::Arc;
use hal::SysError;
use lazy_static::lazy_static;
use num::FromPrimitive;
use rcore_fs::dev::block_cache::BlockCache;
use rcore_fs::vfs::FileSystem;
use rcore_fs::vfs::INode;
use rcore_fs_sfs::SimpleFileSystem;
use requests_info::CastBytes;
use requests_info::fsreqinfo::FsReqDescription;
use spin::Mutex;
use user_syscall::SysResult;

bitflags::bitflags! {
    /// 打开文件时的读写权限
    pub struct OpenFlags: usize {
        /// read only
        const RDONLY = 0;
        /// write only
        const WRONLY = 1 << 0;
        /// read write
        const RDWR = 1 << 1;
        /// create file if it does not exist
        const CREATE = 1 << 6;
        /// error if create and the file exists
        const EXCLUSIVE = 1 << 7;
        /// truncate file upon open
        const TRUNCATE = 1 << 9;
        /// append on each write
        const APPEND = 1<<10;
        /// close on exec
        const CLOEXEC = 1 << 19;
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

    /// Readable
    pub fn readable(&self) -> bool {
        self.readable
    }

    /// Writable
    pub fn writable(&self) -> bool {
        self.writable
    }

    /// Send read request to kthread and wait for service.
    /// If the res > 0, it presents the read size, otherwise the error num.
    pub async fn send_read_req(&self, buf: &mut [u8], fd: usize) -> SysResult {
        if !self.readable() {
            return Err(SysError::EPERM);
        }
        let fs_kthread = KTHREAD_MAP.get().get(&KthreadType::FS);
        match fs_kthread {
            Some(fs_kthread) => {
                let current_thread = current_thread();
                let pid = current_thread.proc().unwrap().pid();
                // 构造fsreq
                let mut res: isize = 0;
                let fsreq = FsReqDescription::Read(
                    pid,
                    fd,
                    buf.as_mut_ptr() as _,
                    buf.len(),
                    &mut res as *mut isize as _,
                )
                .as_bytes()
                .to_vec();
                let fs_kthread = fs_kthread.clone();
                let req_id = fs_kthread.add_request(fsreq);
                current_thread.set_state(ThreadState::Waiting);
                let waitforkthread = WaitForKthread::new(current_thread, fs_kthread, req_id);
                waitforkthread.await;
                match res {
                    size if size >= 0 => return Ok(size as _),
                    err => Err(SysError::from_isize(-err).unwrap()),
                }
            }
            None => {
                error!("[Kernel] Error when sys_open, FS kthread not exist!");
                return Err(SysError::ENOKTH);
            }
        }
    }

    /// Read from inode to buf.
    pub fn read(&self, buf: &mut [u8]) -> SysResult {
        let (mut offset, inode) = (self.offset.lock(), self.inode.lock());
        let n = inode.read_at(*offset, buf)?;
        *offset += n;
        Ok(n)
    }

    /// Send write request to kthread and wait for service.
    /// If the res > 0, it presents the read size, otherwise the error num.
    pub async fn send_write_req(&self, buf: &[u8], fd: usize) -> SysResult {
        let fs_kthread = KTHREAD_MAP.get().get(&KthreadType::FS);
        match fs_kthread {
            Some(fs_kthread) => {
                let current_thread = current_thread();
                let pid = current_thread.proc().unwrap().pid();
                // 构造fsreq
                let mut res: usize = 0;
                let fsreq = FsReqDescription::Write(
                    pid,
                    fd,
                    buf.as_ptr() as _,
                    buf.len(),
                    &mut res as *mut usize as _,
                )
                .as_bytes()
                .to_vec();
                let fs_kthread = fs_kthread.clone();
                let req_id = fs_kthread.add_request(fsreq);
                current_thread.set_state(ThreadState::Waiting);
                let waitforkthread = WaitForKthread::new(current_thread, fs_kthread, req_id);
                waitforkthread.await;
                return Ok(res);
            }
            None => {
                error!("[Kernel] Error when sys_open, FS kthread not exist!");
                return Err(SysError::ENOKTH);
            }
        }
    }

    /// Write data from buf to inode.
    pub fn write(&self, buf: &[u8]) -> SysResult {
        let (mut offset, inode) = (self.offset.lock(), self.inode.lock());
        let n = inode.write_at(*offset, buf)?;
        *offset += n;
        Ok(n)
    }

    /// Lookup from myself.
    pub fn lookup_follow(
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
