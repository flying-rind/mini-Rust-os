//! 驱动模块

use crate::*;
use alloc::sync::Arc;
use block::*;
use lazy_static::lazy_static;
use rcore_fs::dev::{BlockDevice, DevError};
use spin::RwLock;

pub mod block;
mod pci;

lazy_static! {
    pub static ref BLK_DRIVERS: RwLock<Vec<Arc<dyn BlockDriver>>> = RwLock::new(Vec::new());
}

pub struct BlockDriverWrapper(pub Arc<dyn BlockDriver>);

impl BlockDevice for BlockDriverWrapper {
    const BLOCK_SIZE_LOG2: u8 = 9; // 512
    fn read_at(&self, block_id: usize, buf: &mut [u8]) -> rcore_fs::dev::Result<()> {
        match self.0.read_block(block_id, buf) {
            true => Ok(()),
            false => Err(DevError),
        }
    }

    fn write_at(&self, block_id: usize, buf: &[u8]) -> rcore_fs::dev::Result<()> {
        match self.0.write_block(block_id, buf) {
            true => Ok(()),
            false => Err(DevError),
        }
    }

    fn sync(&self) -> rcore_fs::dev::Result<()> {
        Ok(())
    }
}

/// 初始化所有设备驱动程序
pub fn init() {
    pci::init();
}
