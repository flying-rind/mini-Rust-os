//! 内核处理函数

use crate::{MMUFlags, PhysAddr, VirtAddr};
pub trait KernelHandler: Send + Sync + 'static {
    /// Alloc one physical phrame.
    fn frame_alloc(&self) -> Option<PhysAddr> {
        unimplemented!()
    }

    /// Allocate contiguous `frame count` physical frames.
    fn frame_alloc_contihuous(&self) -> Option<PhysAddr> {
        unimplemented!()
    }

    /// Deallocate a physical frame.
    fn frame_dealloc(&self, _paddr: PhysAddr) {
        unimplemented!()
    }

    /// Handle kernel mode page fault.
    fn handle_page_fault(&self, _fault_addr: VirtAddr, _access_flags: MMUFlags) {
        // do nothing
    }
}
