//! 块设备请求处理器

use super::*;
use crate::drivers::BLK_DRIVERS;
use alloc::sync::Arc;
use requests_info::{CastBytes, blkreqinfo::BlkReqDescription};

/// 块设备请求处理器
pub struct BlkProcessor;

impl BlkProcessor {
    /// New.
    pub fn new() -> Arc<Self> {
        Arc::new(BlkProcessor {})
    }

    /// Process read request.
    pub fn process_read(&self, block_id: usize, buf_ptr: usize, buf_len: usize) {
        let buf_ptr = buf_ptr as *mut u8;
        let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, buf_len) };
        let blk_drivers = BLK_DRIVERS.read();
        let blk_driver = blk_drivers.iter().next().unwrap();
        blk_driver.read_block(block_id, buf);
    }

    /// Process write request.
    pub fn process_write(&self, block_id: usize, buf_ptr: usize, buf_len: usize) {
        let buf_ptr = buf_ptr as *const u8;
        let buf = unsafe { core::slice::from_raw_parts(buf_ptr, buf_len) };
        let blk_drivers = BLK_DRIVERS.read();
        let blk_driver = blk_drivers.iter().next().unwrap();
        blk_driver.write_block(block_id, buf);
    }
}

impl Processor for BlkProcessor {
    fn init(&self) {}
    /// 处理一个块设备请求
    fn process_request(&self, request: Request) {
        let blk_req = BlkReqDescription::from_bytes(&request);
        match blk_req {
            BlkReqDescription::ReadBlock(block_id, buf_ptr, buf_len) => {
                self.process_read(*block_id, *buf_ptr, *buf_len);
            }
            BlkReqDescription::WriteBlock(block_id, buf_ptr, buf_len) => {
                self.process_write(*block_id, *buf_ptr, *buf_len);
            }
        }
    }
}
