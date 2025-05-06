//! 块设备驱动程序
pub mod ahci;

/// 块设备驱动程序
pub trait BlockDriver: Send + Sync {
    fn read_block(&self, _block_id: usize, _buf: &mut [u8]) -> bool {
        unimplemented!("not a block driver")
    }

    fn write_block(&self, _block_id: usize, _buf: &[u8]) -> bool {
        unimplemented!("not a block driver")
    }
}
