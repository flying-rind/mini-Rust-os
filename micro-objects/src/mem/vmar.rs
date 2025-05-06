//! 虚拟内存地址区域

use crate::object::KObjectBase;

use super::*;
use crate::impl_kobject;
use crate::object::KoID;
use alloc::{sync::Arc, vec::Vec};
use bitflags::bitflags;
use hal::{GenericPageTable, MMUFlags, VirtAddr};
use spin::Mutex;

bitflags! {
    /// Creation flags for VmAddressRegion.
    pub struct VmarFlags: u32 {
        /// When randomly allocating subregions, reduce sprawl by placing allocations
        /// near each other.
        const COMPACT               = 1 << 0;
        /// Request that the new region be at the specified offset in its parent region.
        const SPECIFIC              = 1 << 1;
        /// Like SPECIFIC, but permits overwriting existing mappings.  This
        /// flag will not overwrite through a subregion.
        const SPECIFIC_OVERWRITE    = 1 << 2;
        /// Allow VmMappings to be created inside the new region with the SPECIFIC or
        /// OFFSET_IS_UPPER_LIMIT flag.
        const CAN_MAP_SPECIFIC      = 1 << 3;
        /// Allow VmMappings to be created inside the region with read permissions.
        const CAN_MAP_READ          = 1 << 4;
        /// Allow VmMappings to be created inside the region with write permissions.
        const CAN_MAP_WRITE         = 1 << 5;
        /// Allow VmMappings to be created inside the region with execute permissions.
        const CAN_MAP_EXECUTE       = 1 << 6;
        /// Require that VMO backing the mapping is non-resizable.
        const REQUIRE_NON_RESIZABLE = 1 << 7;
        /// Treat the offset as an upper limit when allocating a VMO or child VMAR.
        const ALLOW_FAULTS          = 1 << 8;

        /// Allow VmMappings to be created inside the region with read, write and execute permissions.
        const CAN_MAP_RXW           = Self::CAN_MAP_READ.bits | Self::CAN_MAP_EXECUTE.bits | Self::CAN_MAP_WRITE.bits;
        /// Creation flags for root VmAddressRegion
        const ROOT_FLAGS            = Self::CAN_MAP_RXW.bits | Self::CAN_MAP_SPECIFIC.bits;
    }
}

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

/// Virtual Memory Address Regions
pub struct VmAddressRegion {
    flags: VmarFlags,
    base: KObjectBase,
    // _counter: CountHelper,
    addr: VirtAddr,
    size: usize,
    parent: Option<Arc<VmAddressRegion>>,
    page_table: Arc<Mutex<dyn GenericPageTable>>,
    inner: Mutex<Option<VmarInner>>,
}
impl_kobject!(VmAddressRegion);

/// Vmar的可变部分
#[derive(Default)]
struct VmarInner {
    children: Vec<Arc<VmAddressRegion>>,
    mappings: Vec<Arc<VmMapping>>,
}
