//! 内存管理模块
use bootloader_api::info::MemoryRegions;
use buddy_system_allocator::LockedHeap;
use x86_64::structures::paging::PageTableFlags;

use core::fmt;
pub use frame_allocator::*;
pub use memory_set::{MapArea, MemorySet};
pub use page_table::{PageTable, PageTableEntry};

mod frame_allocator;
mod memory_set;
mod page_table;

/// 内存页大小
pub const PAGE_SIZE: usize = 4096;

/// 堆内存页数量
pub const HEAP_PAGES: usize = 1024;

// The virt address of kernel
pub const KERNEL_OFFSET: usize = 0xFFFF_FF00_0000_0000;

// The virt address of kernel_stack
pub const KERNEL_STACK_ADDRESS: usize = 0xFFFF_FF10_0000_0000;
pub const PHYS_OFFSET: usize = 0xFFFF_8000_0000_0000;

// pub const PAGE_SIZE: usize = 4096;
pub const ENTRY_COUNT: usize = 512;

#[global_allocator]
static KERNEL_HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::new();

static mut HEAP_SPACE: [u8; HEAP_PAGES * PAGE_SIZE] = [0; HEAP_PAGES * PAGE_SIZE];
/// 初始化内存管理
pub fn init(memory_regions: &'static mut MemoryRegions) {
    // serial_println!("[dbg] {}.", kernel_base());
    frame_allocator::init(memory_regions);
    page_table::init();
    // 初始化堆内存
    unsafe {
        KERNEL_HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as _, HEAP_PAGES * PAGE_SIZE);
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(transparent)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(transparent)]
pub struct VirtAddr(pub usize);

/// 物理地址转虚拟地址
pub const fn phys_to_virt(pa: usize) -> usize {
    PHYS_OFFSET + pa
}

pub const fn virt_to_phys(va: usize) -> usize {
    va - PHYS_OFFSET
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PA:{:#x}", self.0)
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VA:{:#x}", self.0)
    }
}

impl PhysAddr {
    pub const fn kvaddr(self) -> VirtAddr {
        VirtAddr(phys_to_virt(self.0))
    }
    pub const fn align_down(self) -> Self {
        Self(align_down(self.0))
    }
    pub const fn align_up(self) -> Self {
        Self(align_up(self.0))
    }
    pub const fn page_offset(self) -> usize {
        page_offset(self.0)
    }
    pub const fn is_aligned(self) -> bool {
        is_aligned(self.0)
    }
}

impl VirtAddr {
    pub const fn as_ptr(self) -> *mut u8 {
        self.0 as _
    }
    pub const fn align_down(self) -> Self {
        Self(align_down(self.0))
    }
    pub const fn align_up(self) -> Self {
        Self(align_up(self.0))
    }
    pub const fn page_offset(self) -> usize {
        page_offset(self.0)
    }
    pub const fn is_aligned(self) -> bool {
        is_aligned(self.0)
    }
}

pub const fn align_down(p: usize) -> usize {
    p & !(PAGE_SIZE - 1)
}

pub const fn align_up(p: usize) -> usize {
    (p + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

pub const fn page_offset(p: usize) -> usize {
    p & (PAGE_SIZE - 1)
}

pub const fn is_aligned(p: usize) -> bool {
    page_offset(p) == 0
}
