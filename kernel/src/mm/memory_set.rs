use super::*;
use crate::mm::{page_table::PageTable, PhysFrame, VirtAddr};
use crate::my_x86_64::set_cr3;
use alloc::collections::{btree_map::Entry, BTreeMap};
use core::fmt;
use x86_64::structures::paging::PageTableFlags;

pub struct MapArea {
    pub start: VirtAddr,
    pub size: usize,
    pub flags: PageTableFlags,
    pub mapper: BTreeMap<VirtAddr, PhysFrame>,
}

pub struct MemorySet {
    pub pt: PageTable,
    areas: BTreeMap<VirtAddr, MapArea>,
}

impl MapArea {
    pub fn new(start_va: VirtAddr, size: usize, flags: PageTableFlags) -> Self {
        assert!(start_va.is_aligned() && is_aligned(size));
        Self {
            start: start_va,
            size,
            flags,
            mapper: BTreeMap::new(),
        }
    }

    pub fn clone(&self) -> Self {
        let mut mapper = BTreeMap::new();
        for (&va, old) in &self.mapper {
            let new = PhysFrame::alloc().unwrap();
            new.as_slice().copy_from_slice(old.as_slice());
            mapper.insert(va, new);
        }
        Self {
            start: self.start,
            size: self.size,
            flags: self.flags,
            mapper,
        }
    }

    pub fn map(&mut self, va: VirtAddr) -> PhysAddr {
        assert!(va.is_aligned());
        match self.mapper.entry(va) {
            Entry::Occupied(e) => e.get().start_pa(),
            Entry::Vacant(e) => e.insert(PhysFrame::alloc_zero().unwrap()).start_pa(),
        }
    }

    pub fn unmap(&mut self, va: VirtAddr) {
        self.mapper.remove(&va);
    }

    pub fn write_data(&mut self, offset: usize, data: &[u8]) {
        assert!(offset + data.len() < self.size);
        let mut start = offset;
        let mut remain = data.len();
        let mut processed = 0;
        while remain > 0 {
            let start_align = align_down(start);
            let pgoff = start - start_align;
            let n = (PAGE_SIZE - pgoff).min(remain);
            let pa = self.map(VirtAddr(self.start.0 + start_align));
            unsafe {
                let tmp = pa.kvaddr().as_ptr().add(pgoff);
                core::slice::from_raw_parts_mut(tmp, n)
                    .copy_from_slice(&data[processed..processed + n]);
            }
            start += n;
            processed += n;
            remain -= n;
        }
    }
}

impl MemorySet {
    pub fn new() -> Self {
        Self {
            pt: PageTable::new(),
            areas: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, area: MapArea) {
        if area.size > 0 {
            // TODO: check overlap
            if let Entry::Vacant(e) = self.areas.entry(area.start) {
                self.pt.map_area(e.insert(area));
            } else {
                panic!(
                    "MemorySet::insert: MapArea starts from {:#x?} is existed!",
                    area.start
                );
            }
        }
    }

    pub fn clear(&mut self) {
        for area in self.areas.values_mut() {
            self.pt.unmap_area(area);
        }
        self.areas.clear();
    }

    pub fn activate(&self) {
        set_cr3(self.pt.root_pa.0);
    }
}

impl Clone for MemorySet {
    fn clone(&self) -> Self {
        let mut ms = Self::new();
        for area in self.areas.values() {
            ms.insert(area.clone());
        }
        ms
    }
}

impl Drop for MemorySet {
    fn drop(&mut self) {
        self.clear();
    }
}

impl fmt::Debug for MapArea {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MapArea")
            .field("va_range", &(self.start.0..self.start.0 + self.size))
            .field("flags", &self.flags)
            .finish()
    }
}

impl fmt::Debug for MemorySet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MemorySet")
            .field("areas", &self.areas.values())
            .field("page_table_root", &self.pt.root_pa)
            .finish()
    }
}