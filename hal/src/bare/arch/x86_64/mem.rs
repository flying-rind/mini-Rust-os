//! x86架构的内存相关操作

use core::ops::Range;

use crate::{PhysAddr, PAGE_SIZE};

pub fn free_pmem_regions() -> Vec<Range<PhysAddr>> {
    unimplemented!()
}

/// Flush the physical memory
pub fn frame_flush(target: PhysAddr) {
    unsafe { for paddr in (target..target + PAGE_SIZE).step_by(cacheline_size()) {} }
}
