//! 各硬件平台通用的物理内存操作

use crate::config::KCONFIG;
use crate::hal_fn;
use crate::{PhysAddr, VirtAddr};
use alloc::vec::Vec;
use core::ops::Range;

// 为mem模块的__HalImpl结构实现接口
impl hal_fn::mem::__HalTrait for hal_fn::mem::__HalImpl {
    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        KCONFIG.phys_to_virt_offset + paddr
    }

    fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
        vaddr - KCONFIG.phys_to_virt_offset
    }

    fn free_pmem_regions() -> Vec<Range<PhysAddr>> {
        super::arch::mem::free_pmem_regions()
    }

    fn pmem_read(paddr: PhysAddr, buf: &mut [u8]) {
        let len = buf.len();
        trace!("pmem_read: paddr={:#x}, len={:#x}", paddr, len);
        let src = Self::phys_to_virt(paddr) as *mut u8;
        unsafe {
            buf.as_mut_ptr().copy_from_nonoverlapping(src, len);
        }
    }

    fn pmem_write(paddr: PhysAddr, buf: &[u8]) {
        let len = buf.len();
        trace!("pmem_write: paddr={:#x}, len={:#x}", paddr, len);
        let dst = Self::phys_to_virt(paddr) as *mut u8;
        unsafe {
            dst.copy_from_nonoverlapping(buf.as_ptr(), len);
        }
    }

    fn pmem_zero(paddr: PhysAddr, len: usize) {
        trace!("pmem_zero: paddr={:#x}, len={:#x}", paddr, len);
        unsafe {
            core::ptr::write_bytes(Self::phys_to_virt(paddr) as *mut u8, 0, len);
        }
    }

    fn pmem_copy(dst: PhysAddr, src: PhysAddr, len: usize) {
        trace!("pmem_copy: dst={:#x}, src={:#x}, len={:#x}", dst, src, len);
        let dst = Self::phys_to_virt(dst) as *mut u8;
        unsafe {
            dst.copy_from_nonoverlapping(Self::phys_to_virt(src) as *const u8, len);
        }
    }

    fn frame_flush(target: PhysAddr) {
        super::arch::mem::frame_flush(target)
    }
}
