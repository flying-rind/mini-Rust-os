//! Physical memory operations.

use super::*;
/// 操作物理内存的硬件层接口
pub(crate) trait __HalTrait {
    /// Convert physical address to virtual address.
    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        unimplemented!("mem::phys_to_virt");
    }

    /// Convert virtual address to physical address.
    fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
        unimplemented!("mem::virt_to_phys");
    }

    /// Return all free physical memory regions.
    fn free_pmem_regions() -> Vec<Range<PhysAddr>> {
        unimplemented!("mem::free_pmem_regions");
    }

    /// Read physical memory from `paddr` to `buf`.
    fn pmem_read(paddr: PhysAddr, buf: &mut [u8]) {
        unimplemented!("mem::pmem_read");
    }

    /// Write physical memory from `buf` to `paddr`.
    fn pmem_write(paddr: PhysAddr, buf: &[u8]) {
        unimplemented!("mem::pmem_write");
    }

    /// Zero physical memory at `[paddr, paddr + len]`.
    fn pmem_zero(paddr: PhysAddr, len: usize) {
        unimplemented!("mem::pmem_zero");
    }

    /// Copy content of physical memory `src` to `dst` with `len` bytes.
    fn pmem_copy(dst: PhysAddr, src: PhysAddr, len: usize) {
        unimplemented!("mem::pmem_copy");
    }

    /// Flush the physical frame.
    fn frame_flush(target: PhysAddr) {
        unimplemented!("mem::frame_flush");
    }
}

/// A struct that implements the hal interface.
pub(crate) struct __HalImpl;

// 在模块中定义全局函数将其具体实现导出到`__HalImpl`结构中的实现
fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    __HalImpl::phys_to_virt(paddr)
}

fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    __HalImpl::virt_to_phys(vaddr)
}

fn free_pmem_regions() -> Vec<Range<PhysAddr>> {
    __HalImpl::free_pmem_regions()
}

pub fn pmem_read(paddr: PhysAddr, buf: &mut [u8]) {
    __HalImpl::pmem_read(paddr, buf)
}

fn pmem_write(paddr: PhysAddr, buf: &[u8]) {
    __HalImpl::pmem_write(paddr, buf)
}

fn pmem_zero(paddr: PhysAddr, len: usize) {
    __HalImpl::pmem_zero(paddr, len)
}

/// Copy content of physical memory `src` to `dst` with `len` bytes.
fn pmem_copy(dst: PhysAddr, src: PhysAddr, len: usize) {
    __HalImpl::pmem_copy(dst, src, len)
}

/// Flush the physical frame.
fn frame_flush(target: PhysAddr) {
    __HalImpl::frame_flush(target)
}
