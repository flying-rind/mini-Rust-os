//! 内存管理模块
use bootloader_api::info::MemoryRegions;
pub use frame_allocator::*;
pub use page_table::*;
use x86_64::registers::control::Cr3;
pub use x86_64::structures::paging::PageTableFlags;

mod frame_allocator;
mod heap_allocator;
mod memory_area;
mod memory_set;
mod page_table;
mod physframe;

// mod physical_frame_group;
// mod virtual_memory_block;
// mod virtual_memory_space;

pub use heap_allocator::heap_init;
pub use memory_area::*;
pub use memory_set::*;

/// 内核映射物理内存偏移
pub const PHYS_OFFSET: usize = 0xFFFF_8000_0000_0000;

/// 内存页大小
pub const PAGE_SIZE: usize = 4096;

/// 内核起点的虚拟地址
pub const KERNEL_OFFSET: usize = 0xFFFF_FF00_0000_0000;

/// 内核堆内存大小(4M)
const KERNEL_HEAP_SIZE: usize = 0x0040_0000;

/// 内核堆起始虚拟地址
// const KERNEL_HEAP_BASE: usize = 0xFFFF_FF20_0000_0000;

/// 内核栈虚地址
pub const KERNEL_STACK_BASE: usize = 0xFFFF_FF10_0000_0000;

/// 内核线程栈大小(8M)
pub const KERNEL_STACK_SIZE: usize = 0x80_0000;

/// 用户线程栈大小(4M)
pub const USER_STACK_SIZE: usize = 0x40_0000;

/// 用户栈（最低地址处）
pub const USER_STACK_BASE: usize = 0x0000_7E80_0000_0000;

/// 任意级页表含的页表项个数
pub const ENTRY_COUNT: usize = 512;

/// 初始化内存管理
pub fn init(memory_regions: &'static mut MemoryRegions) {
    frame_allocator::init(memory_regions);
    page_table::init();
}

/// 物理地址转虚拟地址
///
/// （boot程序实际上已经以PHYS_OFFSET的固定偏移将整个物理内存映射过一次了）
pub const fn phys_to_virt(pa: usize) -> usize {
    PHYS_OFFSET + pa
}

/// 虚拟地址转物理地址
pub const fn virt_to_phys(va: usize) -> usize {
    va - PHYS_OFFSET
}

/// 向下对齐，返回当前页的起始地址
pub const fn align_down(p: usize) -> usize {
    p & !(PAGE_SIZE - 1)
}

/// 向上对齐，返回当前页的终止地址
pub const fn align_up(p: usize) -> usize {
    (p + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

/// 获取在当前页中的偏移量
pub const fn page_offset(p: usize) -> usize {
    p & (PAGE_SIZE - 1)
}

/// 是否是关于页对齐的
pub const fn is_aligned(p: usize) -> bool {
    page_offset(p) == 0
}

/// 获取内核态虚存基地址
pub fn vm_kernel_base() -> usize {
    Cr3::read().0.start_address().as_u64() as _
}
