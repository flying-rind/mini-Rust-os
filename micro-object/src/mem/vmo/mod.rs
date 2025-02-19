//! 虚拟内存对象

use super::*;
use crate::object::KoID;
use crate::{error::ZxResult, impl_kobject, object::KObjectBase};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use hal::*;
use spin::Mutex;

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
