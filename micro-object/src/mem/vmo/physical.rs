//! 代表一段物理内存的VMO

use crate::error::ZxResult;
use alloc::sync::Arc;
use hal::{CachePolicy, PhysAddr};
use spin::Mutex;

use super::VMObjectTrait;

/// VMO representing a physical range of memory.
pub struct VMObjectPhysical {
    paddr: PhysAddr,
    pages: usize,
    /// Lock this when access physical memory.
    data_lock: Mutex<()>,
    inner: Mutex<VMObjectPhysicalInner>,
}

/// `VMObjectPhysical`的可变部分
struct VMObjectPhysicalInner {
    cache_policy: CachePolicy,
}

impl VMObjectPhysicalInner {
    pub fn new() -> VMObjectPhysicalInner {
        VMObjectPhysicalInner {
            cache_policy: CachePolicy::Uncached,
        }
    }
}

impl VMObjectPhysical {
    /// Create a new VMO representing a piece of contiguous physical memory.
    /// You must ensure nobody has the ownership of this piece of memory yet.
    pub fn new(paddr: PhysAddr, pages: usize) -> Arc<Self> {
        Arc::new(VMObjectPhysical {
            paddr,
            pages,
            data_lock: Mutex::new(()),
            inner: Mutex::new(VMObjectPhysicalInner::new()),
        })
    }
}

impl VMObjectTrait for VMObjectPhysical {
    fn read(&self, offset: usize, buf: &mut [u8]) -> ZxResult {
        let _ = self.data_lock.lock();
        assert!(offset + buf.len() <= self.len());
        hal::hal_fn::mem::pmem_read()
    }
}
