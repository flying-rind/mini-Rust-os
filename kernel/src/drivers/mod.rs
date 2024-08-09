//! 驱动模块

use crate::*;

use alloc::sync::Arc;

mod ahci;
mod pci;

/// 全局块设备
pub static BLOCK_DEVICE: Cell<Arc<dyn BlockDevice>> =
    unsafe { transmute(&0 as *const _ as *const ahci::AHCIDriver as *const dyn BlockDevice) };

pub fn init() {
    unsafe {
        (BLOCK_DEVICE.get_mut() as *mut Arc<dyn BlockDevice>).write(Arc::new(pci::init().unwrap()));
    }
}
