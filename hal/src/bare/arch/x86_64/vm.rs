//! x86_64 virtual memory operations.
use crate::hal_fn::vm::*;

impl __HalTrait for __HalImpl {
    fn current_vmtoken() -> crate::PhysAddr {
        unimplemented!()
    }

    fn activate_paging(vmtoken: crate::PhysAddr) {
        unimplemented!()
    }

    fn flush_tlb(vaddr: Option<crate::VirtAddr>) {
        unimplemented!()
    }

    fn pt_clone_kernel_space(dst_pt_root: crate::PhysAddr, src_pt_root: crate::PhysAddr) {
        unimplemented!()
    }
}
