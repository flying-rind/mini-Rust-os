//! 物理页帧初始化

use bitmap_allocator::{BitAlloc, BitAlloc256M};
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use spin::Mutex;

use super::PAGE_SIZE;

/// 使用bitmap_allocator库定义全局BIT_ALLOCATOR
static BIT_ALLOCATOR: Mutex<BitAlloc256M> = Mutex::new(BitAlloc256M::DEFAULT);

/// 初始化页帧分配器
pub fn init(memory_regions: &'static mut MemoryRegions) {
    let mut ba = BIT_ALLOCATOR.lock();
    serial_println!("[Kernel] Memory regions:");
    for region in memory_regions.into_iter() {
        serial_println!("    {:x?}", region);
        if region.kind == MemoryRegionKind::Usable {
            let start = region.start as usize;
            let end = region.end;
            let start_frame = start as usize / PAGE_SIZE;
            let end_frame = end as usize / PAGE_SIZE;
            ba.insert(start_frame..end_frame);
        }
    }
}

/// 申请一个可用页帧，返回首地址
pub fn allocate_frame() -> Option<usize> {
    let mut ba = BIT_ALLOCATOR.lock();
    ba.alloc().map(|id| id * PAGE_SIZE)
}

/// 申请一组连续的页帧，返回第一个页帧的首地址
pub fn allocate_frame_cont(size: usize) -> Option<usize> {
    let mut ba = BIT_ALLOCATOR.lock();
    ba.alloc_contiguous(size, 0).map(|id| id * PAGE_SIZE)
}

/// 释放给定地址的物理页帧
pub fn deallocate_frame(frame: usize) {
    let mut ba = BIT_ALLOCATOR.lock();
    ba.dealloc(frame / PAGE_SIZE)
}

// 拿来主义

use crate::mem::PhysAddr;

#[derive(Debug)]
#[repr(transparent)]
pub struct PhysFrame(usize);

impl PhysFrame {
    pub const fn start_pa(&self) -> PhysAddr {
        // PhysAddr(self.0.get())
        PhysAddr(self.0)
    }

    pub fn alloc() -> Option<Self> {
        let mut ba = BIT_ALLOCATOR.lock();
        let paddr = ba.alloc().map(|id| id * PAGE_SIZE).unwrap();
        Some(PhysFrame(paddr))
    }

    pub fn dealloc(pa: usize) {
        BIT_ALLOCATOR.lock().dealloc(pa)
    }

    pub fn alloc_zero() -> Option<Self> {
        let mut f = Self::alloc()?;
        f.zero();
        Some(f)
    }

    pub fn zero(&mut self) {
        unsafe { core::ptr::write_bytes(self.start_pa().kvaddr().as_ptr(), 0, PAGE_SIZE) }
    }

    pub fn as_slice(&self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.start_pa().kvaddr().as_ptr(), PAGE_SIZE) }
    }
}

impl Drop for PhysFrame {
    fn drop(&mut self) {
        BIT_ALLOCATOR.lock().dealloc(self.0);
    }
}
