//! Virtual memory operations.
use super::*;

pub(crate) trait __HalTrait {
    /// Read the current VM token, which is the page table root address on
    /// various architectures. (e.g. CR3, SATP...)
    fn current_vmtoken() -> PhysAddr {
        unimplemented!("vm::current_vmtoken");
    }

    /// Activate the page table associated with the `vmtoken` by writting the
    /// page table root address.
    fn activate_paging(vmtoken: PhysAddr) {
        unimplemented!("vm::activate_paging");
    }

    /// Flush TLB by the associated `vaddr`, or flush the entire TLB.(`vaddr` is `None`).
    fn flush_tlb(vaddr: Option<VirtAddr>) {
        unimplemented!("vm::flush_tlb");
    }

    /// Clone kernel space entries(top level only) from `src` page table to `dst` page table.
    fn pt_clone_kernel_space(dst_pt_root: PhysAddr, src_pt_root: PhysAddr) {
        unimplemented!("vm::pt_clone_kernel_space");
    }
}

/// A struct that implements the hal interface.
pub(crate) struct __HalImpl;

/// Read the current VM token, which is the page table root address on
/// various architectures. (e.g. CR3, SATP...)
pub fn current_vmtoken() -> PhysAddr {
    __HalImpl::current_vmtoken()
}

/// Activate the page table associated with the `vmtoken` by writting the
/// page table root address.
pub fn activate_paging(vmtoken: PhysAddr) {
    __HalImpl::activate_paging(vmtoken);
}

/// Flush TLB by the associated `vaddr`, or flush the entire TLB.(`vaddr` is `None`).
pub fn flush_tlb(vaddr: Option<VirtAddr>) {
    __HalImpl::flush_tlb(vaddr);
}

/// Clone kernel space entries(top level only) from `src` page table to `dst` page table.
pub fn pt_clone_kernel_space(dst_pt_root: PhysAddr, src_pt_root: PhysAddr) {
    __HalImpl::pt_clone_kernel_space(dst_pt_root, src_pt_root);
}
