//! 全局物理页帧分配器

use super::*;
use crate::*;
use bitmap_allocator::{BitAlloc, BitAlloc256M};
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use spin::mutex::Mutex;

use super::PAGE_SIZE;

/// 使用bitmap_allocator库定义全局BIT_ALLOCATOR
static BIT_ALLOCATOR: Mutex<BitAlloc256M> = Mutex::new(BitAlloc256M::DEFAULT);

/// 初始化页帧分配器
pub fn init(memory_regions: &'static mut MemoryRegions) {
    let mut ba = BIT_ALLOCATOR.lock();
    println!("[Kernel] Memory regions:");
    for region in memory_regions.into_iter() {
        println!("    {:x?}", region);
        if region.kind == MemoryRegionKind::Usable {
            let start = region.start as usize;
            let end = region.end;
            let start_frame = start as usize / PAGE_SIZE;
            let end_frame = end as usize / PAGE_SIZE;
            ba.insert(start_frame..end_frame);
        }
    }
}

/// 申请一个可用页帧，返回首物理地址
pub fn allocate_frame() -> Option<usize> {
    let mut ba = BIT_ALLOCATOR.lock();
    let paddr = ba.alloc().map(|id| id * PAGE_SIZE);
    if let Some(paddr) = paddr {
        Some(paddr)
    } else {
        error!("[Kernel] Fail to allocate frame");
        None
    }
}
/// 释放给定地址的物理页帧
pub fn deallocate_frame(frame: usize) {
    let mut ba = BIT_ALLOCATOR.lock();
    ba.dealloc(frame / PAGE_SIZE);
}

/// 清零页帧
pub fn clear_frames(paddr: usize, pages: usize) {
    unsafe { core::ptr::write_bytes((phys_to_virt(paddr)) as *mut u8, 0, PAGE_SIZE * pages) };
}
