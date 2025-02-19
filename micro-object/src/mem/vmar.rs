//! 虚拟内存地址区域

use super::*;
use alloc::{sync::Arc, vec::Vec};
use hal::{GenericPageTable, MMUFlags, VirtAddr};
use spin::Mutex;

/// Virtual Memory Mapping
pub struct VmMapping {
    /// The permission limitation of the vmar
    permissions: MMUFlags,
    vmo: Arc<VmObject>,
    page_table: Arc<Mutex<dyn GenericPageTable>>,
    inner: Mutex<VmMappingInner>,
}

/// VmMapping的可变部分
#[derive(Debug, Clone)]
struct VmMappingInner {
    flags: Vec<MMUFlags>,
    addr: VirtAddr,
    size: usize,
    vmo_offset: usize,
}
