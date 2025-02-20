//! 虚拟内存对象

use super::*;
use crate::error::ZxError;
use crate::object::*;
use crate::{error::ZxResult, impl_kobject, object::KObjectBase};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use bitflags::bitflags;
use hal::*;
use spin::Mutex;
use spin::MutexGuard;

mod paged;

/// Virtual Memory Object Trait
pub trait VMObjectTrait: Sync + Send {
    /// Read memory to `buf` from VMO at `offset`.
    fn read(&self, offset: usize, buf: &mut [u8]) -> ZxResult;

    /// Write memory from `buf` to VMO at `offset`.
    fn write(&self, offset: usize, buf: &[u8]) -> ZxResult;

    /// Reset the range of bytes in the VMO from `offset` to `offset+len` to 0.
    fn zero(&self, offset: usize, len: usize) -> ZxResult;

    /// Get the length of the VMO.
    fn len(&self) -> usize;

    /// Set the length of the VMO.
    fn set_len(&self, len: usize) -> ZxResult;

    /// Commit a page.
    fn commit_page(&self, page_idx: usize, flags: MMUFlags) -> ZxResult<PhysAddr>;

    /// Commit pages with an external function f.
    /// the vmo is internally locked before it calls f,
    /// allowing `VmMapping` to avoid deadlock
    fn commit_pages_with(
        &self,
        f: &mut dyn FnMut(&mut dyn FnMut(usize, MMUFlags) -> ZxResult<PhysAddr>) -> ZxResult,
    ) -> ZxResult;

    /// Commit allocating physical memory.
    fn commit(&self, offset: usize, len: usize) -> ZxResult;

    /// Decommit allocated physical memory.
    fn decommit(&self, offset: usize, len: usize) -> ZxResult<Arc<dyn VMObjectTrait>>;

    /// Create a child VMO
    fn create_child(&self, offset: usize, len: usize) -> ZxResult<Arc<dyn VMObjectTrait>>;

    /// Append a mapping to the VMO's mapping list.
    fn append_mapping(&self, _mapping: Weak<VmMapping>) {}

    /// Remove a mapping from the VMO's mapping list.
    fn remove_mapping(&self, _mapping: Weak<VmMapping>) {}

    /// Complete the VmoInfo.
    fn complete_info(&self, info: &mut VmoInfo);

    // /// Get the cache policy.
    // fn cache_policy(&self) -> CachePolicy;

    // /// Set the cache policy.
    // fn set_cache_policy(&self, policy: CachePolicy) -> ZxResult;

    /// Count committed pages of the VMO.
    fn committed_pages_in_range(&self, start_idx: usize, end_idx: usize) -> usize;

    /// Pin the given range of the VMO.
    fn pin(&self, _offset: usize, _len: usize) -> ZxResult {
        Err(ZxError::NOT_SUPPORTED)
    }

    /// Unpin the given range of the VMO.
    fn unpin(&self, _offset: usize, _len: usize) -> ZxResult {
        Err(ZxError::NOT_SUPPORTED)
    }

    /// Returns true if the object is backed by a contiguous range of physical memory.
    fn is_contiguous(&self) -> bool {
        false
    }

    /// Returns true if the object is backed by RAM.
    fn is_paged(&self) -> bool {
        false
    }

    /// If contiguous, transmute vmo to a mutable buffer
    fn as_mut_buf(&self) -> ZxResult<(MutexGuard<()>, &mut [u8])> {
        Err(ZxError::NOT_SUPPORTED)
    }

    /// Mark as not contiguous
    fn unset_contiguous(&self) {}
}

/// Virtual memory containers
///
/// ## SYNOPSIS
///
/// A Virtual Memory Object(VMO) represents a contiguous region of virtual memory
/// that may be mapped into multiple address spaces.
pub struct VmObject {
    base: KObjectBase,
    // _counter: CounterHelper,
    resizable: bool,
    trait_: Arc<dyn VMObjectTrait>,
    inner: Mutex<VmObjectInner>,
}

impl_kobject!(VmObject);

/// VMObject的可变部分
#[derive(Default)]
struct VmObjectInner {
    /// 父对象
    parent: Weak<VmObject>,
    /// 子对象
    children: Vec<Weak<VmObject>>,
    /// 映射数
    mapping_count: usize,
    /// 内容大小
    content_size: usize,
}

bitflags! {
    #[derive(Default)]
    /// Values used by ZX_INFO_PROCESS_VMOS.
    pub struct VmoInfoFlags: u32 {
        /// The VMO points to a physical address range, and does not consume memory.
        /// Typically used to access memory-mapped hardware.
        /// Mutually exclusive with TYPE_PAGED.
        const TYPE_PHYSICAL = 0;

        #[allow(clippy::identity_op)]
        /// The VMO is backed by RAM, consuming memory.
        /// Mutually exclusive with TYPE_PHYSICAL.
        const TYPE_PAGED    = 1 << 0;

        /// The VMO is resizable.
        const RESIZABLE     = 1 << 1;

        /// The VMO is a child, and is a copy-on-write clone.
        const IS_COW_CLONE  = 1 << 2;

        /// When reading a list of VMOs pointed to by a process, indicates that the
        /// process has a handle to the VMO, which isn't necessarily mapped.
        const VIA_HANDLE    = 1 << 3;

        /// When reading a list of VMOs pointed to by a process, indicates that the
        /// process maps the VMO into a VMAR, but doesn't necessarily have a handle to
        /// the VMO.
        const VIA_MAPPING   = 1 << 4;

        /// The VMO is a pager owned VMO created by zx_pager_create_vmo or is
        /// a clone of a VMO with this flag set. Will only be set on VMOs with
        /// the ZX_INFO_VMO_TYPE_PAGED flag set.
        const PAGER_BACKED  = 1 << 5;

        /// The VMO is contiguous.
        const CONTIGUOUS    = 1 << 6;
    }
}

/// Describes a VMO
#[repr(C)]
#[derive(Default)]
pub struct VmoInfo {
    /// The KoID of this VMO.
    koid: KoID,
    /// The name of this VMO.
    name: [u8; 32],
    /// The size of this VMO; i.e., the amount of virtual address space it
    /// would comsume if mapped.
    size: u64,
    /// If this VMO is cloned, the koid of it parent. Otherwise, zero.
    parent_koid: KoID,
    /// The number of clones of this VMO, if any.
    num_children: u64,
    /// The number of times this VMO is currently mapped into VMARs.
    num_mappings: u64,
    /// The number of unique address space we're mapped into.
    share_count: u64,
    /// Flags.
    pub flags: VmoInfoFlags,
    /// Padding.
    padding1: [u8; 4],
    /// If the type is `PAGED`, the amount of
    /// memory currently allocated to this VMO; i.e., the amount of physical
    /// memory it consumes. Undefined otherwise.
    committed_bytes: u64,
    /// If `flags & ZX_INFO_VMO_VIA_HANDLE`, the handle rights.
    /// Undefined otherwise.
    pub rights: Rights,
    /// VMO mapping cache policy.
    cache_policy: u32,
}
