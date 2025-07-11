//! 文件系统内核线程的响应器

use core::{
    panic,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{activate_proc_ms, fs::File, task::PROCESS_MAP};

use super::*;
use alloc::sync::Arc;
use requests_info::{CastBytes, fsreqinfo::FsReqDescription};

/// 文件系统请求处理器
pub struct FsProcessor;

impl FsProcessor {
    pub fn new() -> Arc<Self> {
        Arc::new(FsProcessor {})
    }
}

/// 测试
pub static PROCESSED_COUNT: AtomicUsize = AtomicUsize::new(0);

impl FsProcessor {
    /// Process Read request. If the res > 0, it presents the read size, otherwise the error num.
    pub fn process_read(
        &self,
        pid: usize,
        fd: usize,
        buf_ptr: usize,
        buf_len: usize,
        res_ptr: usize,
    ) {
        activate_proc_ms(pid.clone());
        let buf_ptr = buf_ptr as *mut u8;
        let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, buf_len) };
        let proc = PROCESS_MAP.get().get(&pid);
        assert!(proc.is_some());
        let proc = proc.unwrap();
        let file = proc.file_table().get(&fd).unwrap();
        let read_size: isize = match file.as_ref() {
            File::OSInode(osinode) => match osinode.read(buf) {
                Ok(size) => size as _,
                Err(err) => -(err as isize),
            },
            _ => panic!("File type error!"),
        };
        let result_ptr = res_ptr as *mut isize;
        unsafe { *result_ptr = read_size };
    }

    /// Process Write request. If the res > 0, it presents the read size, otherwise the error num.
    pub fn process_write(
        &self,
        pid: usize,
        fd: usize,
        buf_ptr: usize,
        buf_len: usize,
        res_ptr: usize,
    ) {
        activate_proc_ms(pid.clone());
        // [模拟致命错误]
        if PROCESSED_COUNT.load(Ordering::Relaxed) % 5 == 0 {
            PROCESSED_COUNT.fetch_add(1, Ordering::Relaxed);
            panic!("[Fs Processor] Fatal error in write request!");
        }
        let buf_ptr = buf_ptr as *const u8;
        let buf = unsafe { core::slice::from_raw_parts(buf_ptr, buf_len) };
        let proc = PROCESS_MAP.get().get(&pid);
        assert!(proc.is_some());
        let proc = proc.unwrap();
        let file = proc.file_table().get(&fd).unwrap();
        let write_size: isize = match file.as_ref() {
            File::OSInode(osinode) => match osinode.write(buf) {
                Ok(size) => size as _,
                Err(err) => -(err as isize),
            },
            _ => panic!(""),
        };
        let res_ptr = res_ptr as *mut isize;
        unsafe {
            *res_ptr = write_size;
        }
    }
}

impl Processor for FsProcessor {
    /// 处理一个文件系统请求，并不响应请求（在内核入口函数中响应）
    fn process_request(&self, request: Request) {
        let fs_req = FsReqDescription::from_bytes(&request);
        match fs_req {
            // Read请求，进程Pid读文件表中fd对应的文件到buf中
            FsReqDescription::Read(pid, fd, buf_ptr, buf_len, res_ptr) => {
                self.process_read(*pid, *fd, *buf_ptr, *buf_len, *res_ptr);
            }
            // Write请求，进程Pid将buf中的数据写入文件表中fd对应的文件
            FsReqDescription::Write(pid, fd, buf_ptr, buf_len, res_ptr) => {
                self.process_write(*pid, *fd, *buf_ptr, *buf_len, *res_ptr);
            }
        }
        PROCESSED_COUNT.fetch_add(1, Ordering::Relaxed);
    }
}
