//! 各平台通用的内存抽象模块

use super::MMUFlags;

pub type PhysAddr = usize;
pub type VirtAddr = usize;

/// Errors may occur during address translation.
#[derive(Debug)]
pub enum PagingError {
    NoMemory,
    NotMapped,
    AlreadyMapped,
}

/// Address translation result.
pub type PagingResult<T = ()> = Result<T, PagingError>;

/// Possible page size (4K, 2M, 1G).
#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PageSize {
    Size4K = 0x1000,
    Size2M = 0x20_0000,
    Size1G = 0x4000_0000,
}

/// A 4k, 2M or 1G size page.
#[derive(Debug, Copy, Clone)]
pub struct Page {
    pub vaddr: usize,
    pub size: PageSize,
}

/// A generic page table abstraction
pub trait GenericPageTable: Send + Sync {
    /// Get the physical address of root page table.
    fn table_phys(&self) -> PhysAddr;

    /// Map the `page` to the frame of `paddr` with `flags`.
    fn map(&mut self, page: Page, paddr: PhysAddr, flags: MMUFlags) -> PagingResult;

    /// Unmap the page of `vaddr`.
    fn unmap(&mut self, vaddr: VirtAddr) -> PagingResult<(PhysAddr, PageSize)>;

    /// Change the `flags` of the page of `vaddr`.
    fn update(
        &mut self,
        vaddr: VirtAddr,
        paddr: Option<PhysAddr>,
        flags: Option<MMUFlags>,
    ) -> PagingResult<PageSize>;

    /// Query the physical address which the page of `vaddr` maps to.
    fn query(&self, vaddr: VirtAddr) -> PagingResult<(PhysAddr, MMUFlags, PageSize)>;

    fn map_cont(
        &mut self,
        start_vaddr: VirtAddr,
        size: usize,
        start_paddr: PhysAddr,
        flags: MMUFlags,
    ) -> PagingResult {
        unimplemented!()
    }

    fn unmap_cont(&mut self, start_vaddr: VirtAddr, size: usize) -> PagingResult {
        unimplemented!()
    }
}
