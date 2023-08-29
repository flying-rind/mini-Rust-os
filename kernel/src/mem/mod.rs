//! 内存管理模块

use core::usize;

// use crate::mem::page_table::PageTable;

use self::page_table::PageTableOldv;
use bootloader_api::info::MemoryRegions;
use buddy_system_allocator::LockedHeap;
use x86_64::{registers::control::Cr3, structures::paging::PageTableFlags};

mod frame_allocator;
pub mod memory_set;
mod page_table;

pub use frame_allocator::*;

/// 内存页大小
pub const PAGE_SIZE: usize = 4096;

/// 堆内存页数量
pub const HEAP_PAGES: usize = 1024;

/// 堆内存基地至
pub const HEAP_BASE: usize = 0xFFFF_FF00_0000_0000;

/// 物理地址偏移
pub const KERNEL_PHY_OFFSET: usize = 0xFFFF_FF80_0000_0000;

/// 物理地址转虚拟地址
pub const fn phys_to_virt(paddr: usize) -> usize {
    KERNEL_PHY_OFFSET + paddr
}

#[global_allocator]
static KERNEL_HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::new();

static mut HEAP_SPACE: [u8; HEAP_PAGES * PAGE_SIZE] = [0; HEAP_PAGES * PAGE_SIZE];
/// 初始化内存管理
pub fn init(memory_regions: &'static mut MemoryRegions) {
    // serial_println!("[dbg] {}.", kernel_base());

    init_frame_allocator(memory_regions);

    // 分配物理页帧作为内核堆内存
    // let heap_frame = allocate_frame_cont(HEAP_PAGES).expect("Failed to allocate frame");
    // let page_table = PageTableOldv::new(kernel_base());
    page_table::init();
    // let mut page_table = PageTable::new();
    // let flags =
    //     PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
    // let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    // 将堆内存的每个页帧映射到相应虚拟地址
    // serial_println!(
    //     "[Kernel] Map vaddr 0x{:x} to paddr 0x{:x} ({:?} pages in total)",
    //     HEAP_BASE,
    //     heap_frame,
    //     HEAP_PAGES
    // );
    // for i in 0..HEAP_PAGES {
    //     let vaddr = VirtAddr(HEAP_BASE + i * PAGE_SIZE);
    //     let paddr = PhysAddr(heap_frame + i * PAGE_SIZE);
    //     page_table.map(vaddr, paddr, flags);
    // }

    // 初始化堆内存
    unsafe {
        KERNEL_HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as _, HEAP_PAGES * PAGE_SIZE);
    }
}

/// 获取内核态虚存基地址
pub fn kernel_base() -> usize {
    Cr3::read().0.start_address().as_u64() as _
}

pub fn set_pagetable_to_user() {
    let root = kernel_base();
    // let l0entr = unsafe { core::ptr::read(l0addr as *mut u64) };
    unsafe {
        for i in 0..512 {
            let l1entry = core::ptr::read((root + i * 8 + KERNEL_PHY_OFFSET) as *mut usize);
            if l1entry == 0 {
                continue;
            } else {
                core::ptr::write(
                    (root + i * 8 + KERNEL_PHY_OFFSET) as *mut usize,
                    0x7fff_ffff_ffff_ffff & l1entry
                        | PageTableFlags::USER_ACCESSIBLE.bits() as usize,
                );
                let l2addr = (l1entry >> 12) << 12;
                for j in 0..512 {
                    let l2entry =
                        core::ptr::read((l2addr + j * 8 + KERNEL_PHY_OFFSET) as *mut usize);
                    if l2entry == 0 {
                        continue;
                    } else {
                        core::ptr::write(
                            (l2addr + j * 8 + KERNEL_PHY_OFFSET) as *mut usize,
                            0x7fff_ffff_ffff_ffff & l2entry
                                | PageTableFlags::USER_ACCESSIBLE.bits() as usize,
                        );
                        let l3addr = (l2entry >> 12) << 12;
                        for k in 0..512 {
                            let l3entry =
                                core::ptr::read((l3addr + k * 8 + KERNEL_PHY_OFFSET) as *mut usize);
                            if l3entry == 0 {
                                continue;
                            } else {
                                core::ptr::write(
                                    (l3addr + k * 8 + KERNEL_PHY_OFFSET) as *mut usize,
                                    0x7fff_ffff_ffff_ffff & l3entry
                                        | PageTableFlags::USER_ACCESSIBLE.bits() as usize,
                                );
                                let l4addr = (l3entry >> 12) << 12;
                                for l in 0..512 {
                                    let l4entry = core::ptr::read(
                                        (l4addr + l * 8 + KERNEL_PHY_OFFSET) as *mut usize,
                                    );
                                    if l4entry == 0 {
                                        continue;
                                    } else {
                                        core::ptr::write(
                                            (l4addr + l * 8 + KERNEL_PHY_OFFSET) as *mut usize,
                                            0x7fff_ffff_ffff_ffff & l4entry
                                                | PageTableFlags::USER_ACCESSIBLE.bits() as usize,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test_case]
fn test_heap() {
    alloc::vec![1, 2, 3, 4, 5, 6];
}

// 拿来主义

use core::fmt;

pub const KERNEL_OFFSET: usize = 0x0000008000000000;
pub const PHYS_OFFSET: usize = 0xFFFF_FF80_0000_0000;

// pub const PAGE_SIZE: usize = 4096;
pub const ENTRY_COUNT: usize = 512;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(transparent)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(transparent)]
pub struct VirtAddr(pub usize);

// pub const fn phys_to_virt(pa: usize) -> usize {
//     pa + PHYS_OFFSET
// }

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
