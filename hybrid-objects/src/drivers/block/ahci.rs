//! AHCI设备驱动程序

use super::BlockDriver;
use crate::{drivers::BLK_DRIVERS, *};
use alloc::sync::Arc;
use isomorphic_drivers::{
    block::ahci::{AHCI, BLOCK_SIZE},
    provider,
};
use mm::{allocate_frame, deallocate_frame};
use spin::Mutex;

/// AHCI设备驱动程序
pub struct AHCIDriver(Mutex<AHCI<Provider>>);

impl AHCIDriver {
    pub fn new(header: usize, size: usize) -> Option<Self> {
        AHCI::new(header, size).map(|x| Self(Mutex::new(x)))
    }
}

impl BlockDriver for AHCIDriver {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> bool {
        let mut driver = self.0.lock();
        driver.read_block(block_id, buf);
        true
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) -> bool {
        if buf.len() < BLOCK_SIZE {
            return false;
        }
        let mut driver = self.0.lock();
        driver.write_block(block_id, buf);
        true
    }
}

/// 初始化AHCI设备驱动
pub fn init(header: usize, size: usize) -> Option<Arc<AHCIDriver>> {
    if let Some(ahcidriver) = AHCIDriver::new(header, size) {
        let driver = Arc::new(ahcidriver);
        // 写入全局变量
        BLK_DRIVERS.write().push(driver.clone());
        Some(driver)
    } else {
        None
    }
}

struct Provider;

impl provider::Provider for Provider {
    const PAGE_SIZE: usize = mm::PAGE_SIZE;

    fn alloc_dma(size: usize) -> (usize, usize) {
        println!("alloc_dma: {:x}", size);
        let pages = size / mm::PAGE_SIZE;
        let mut base = 0;
        for i in 0..pages {
            let frame = allocate_frame().unwrap();
            // let frame_pa = frame.start_pa().0;
            // core::mem::forget(frame);
            if i == 0 {
                base = frame;
            }
            assert_eq!(frame, base + i * mm::PAGE_SIZE);
        }
        println!("virtio_dma_alloc: {:x} {}", base, pages);
        (mm::phys_to_virt(base), base)
    }

    fn dealloc_dma(va: usize, size: usize) {
        println!("dealloc_dma: {:x} {:x}", va, size);
        let pages = size / mm::PAGE_SIZE;
        let mut pa = mm::virt_to_phys(va);
        for _ in 0..pages {
            deallocate_frame(pa);
            pa += mm::PAGE_SIZE;
        }
    }
}
