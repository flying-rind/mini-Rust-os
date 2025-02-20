//! 代表一段物理内存的VMO

use hal::{CachePolicy, PhysAddr};
use spin::Mutex;

/// VMO representing a physical range of memory.
pub struct VMObjectPhysical {
    paddr: PhysAddr,
    pages: usize,
    /// Lock this when access physical memory.
    inner: Mutex<VMObjectPhysicalInner>,
}

/// `VMObjectPhysical`的可变部分
struct VMObjectPhysicalInner {
    cache_policy: CachePolicy,
}
