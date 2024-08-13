//! 文件系统内核线程的响应器

use core::{
    panic,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{activate_proc_ms, println, task::PROCESS_MAP};

use super::*;
use crate::fs::*;
use alloc::sync::Arc;
use requests_info::{fsreqinfo::FsReqDescription, CastBytes};

/// 文件系统请求处理器
pub struct FsProcessor;

impl FsProcessor {
    pub fn new() -> Arc<Self> {
        Arc::new(FsProcessor {})
    }
}

/// 测试
pub static PROCESSED_COUNT: AtomicUsize = AtomicUsize::new(0);

impl Processor for FsProcessor {
    /// 处理一个文件系统请求，并不响应请求（在内核入口函数中响应）
    fn process_request(&self, request: Request) {
        let fs_req = FsReqDescription::from_bytes(&request);
        match fs_req {
            // Read请求，进程Pid读文件表中fd对应的文件到buf中
            FsReqDescription::Read(pid, fd, buf_ptr, buf_len, result_ptr) => {
                // 切换到进程所在地址空间
                activate_proc_ms(pid.clone());
                // 解析buf
                let buf_ptr = (*buf_ptr) as *mut u8;
                let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, *buf_len) };
                // 获取磁盘文件
                let proc = PROCESS_MAP.get().get(pid);
                assert!(proc.is_some());
                let proc = proc.unwrap();
                // sys_read中已保证file存在文件表中
                let file = proc
                    .file_table()
                    .get(*fd)
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .clone();
                let read_size = if !file.readable() {
                    println!("[Fs server] Error reading file, not readable!");
                    usize::MAX
                } else {
                    file.read(buf)
                };
                // 将read_size写入到用户态中
                let result_ptr = (*result_ptr) as *mut usize;
                unsafe {
                    *result_ptr = read_size;
                }
            }
            // Write请求，进程Pid将buf中的数据写入文件表中fd对应的文件
            FsReqDescription::Write(pid, fd, buf_ptr, buf_len, result_ptr) => {
                // 切换到进程所在地址空间
                activate_proc_ms(pid.clone());
                // [模拟致命错误]
                if PROCESSED_COUNT.load(Ordering::Relaxed) % 5 == 0 {
                    PROCESSED_COUNT.fetch_add(1, Ordering::Relaxed);
                    panic!("[Fs Processor] Fatal error in write request!");
                }

                // 解析buf
                let buf_ptr = (*buf_ptr) as *const u8;
                let buf = unsafe { core::slice::from_raw_parts(buf_ptr, *buf_len) };
                // 获取磁盘文件
                let proc = PROCESS_MAP.get().get(pid);
                assert!(proc.is_some());
                let proc = proc.unwrap();
                // sys_read中已保证file存在文件表中
                let file = proc
                    .file_table()
                    .get(*fd)
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .clone();
                let write_size = if !file.writable() {
                    println!("[Fs server] Error writing file, not readable!");
                    usize::MAX
                } else {
                    file.write(buf)
                };
                // 将read_size写入到用户态中
                let result_ptr = (*result_ptr) as *mut usize;
                unsafe {
                    *result_ptr = write_size;
                }
            }
            // 处理Open请求，打开一个磁盘文件并加入pid进程的文件表中
            FsReqDescription::Open(pid, path_ptr, flags, fd_ptr) => {
                // 切换到进程所在地址空间
                activate_proc_ms(pid.clone());
                // 解析路径
                let flags = OpenFlags::from_bits(*flags).unwrap();
                let path_ptr = (*path_ptr) as *const &str;
                let path = unsafe { *path_ptr };
                let (readable, writable) = flags.read_write();
                // 可以创建文件
                let file = if flags.contains(OpenFlags::CREATE) {
                    // 文件已存在
                    if let Some(inode) = ROOT_INODE.find(path) {
                        // clear size
                        inode.clear();
                        Some(Arc::new(OSInode::new(readable, writable, inode)))
                    // 不存在，需要创建
                    } else {
                        ROOT_INODE
                            .create(path)
                            .map(|inode| Arc::new(OSInode::new(readable, writable, inode)))
                    }
                // 不能创建文件
                } else {
                    ROOT_INODE.find(path).map(|inode| {
                        if flags.contains(OpenFlags::TRUNC) {
                            inode.clear();
                        }
                        Arc::new(OSInode::new(readable, writable, inode))
                    })
                };
                let fd = match file {
                    Some(file) => {
                        // 将文件添加到进程文件表中
                        let process = PROCESS_MAP.get().get(&*pid);
                        assert!(process.is_some());
                        process.unwrap().add_file(file)
                    }
                    None => {
                        println!(
                            "[Fs Kthread] Open request failed, cannot open file with path {}",
                            path
                        );
                        // 未能打开文件时返回usize::MAX
                        usize::MAX
                    }
                };
                // 将结果写入fd指针
                let fd_ptr = (*fd_ptr) as *mut usize;
                unsafe {
                    *fd_ptr = fd;
                }
            }
        }
        PROCESSED_COUNT.fetch_add(1, Ordering::Relaxed);
    }
}
