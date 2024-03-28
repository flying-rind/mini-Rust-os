use super::*;
use crate::*;
use core::fmt;

static KERNEL_PTE: Cell<PageTableEntry> = zero();
static PHYS_PTE: Cell<PageTableEntry> = zero();

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    const PHYS_ADDR_MASK: usize = !(PAGE_SIZE - 1);

    pub const fn empty() -> Self {
        Self(0)
    }
    pub const fn new_page(pa: PhysAddr, flags: PageTableFlags) -> Self {
        Self((pa.0 & Self::PHYS_ADDR_MASK) | flags.bits() as usize)
    }
    const fn pa(self) -> PhysAddr {
        PhysAddr(self.0 as usize & Self::PHYS_ADDR_MASK)
    }
    const fn flags(self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.0 as _)
    }
    const fn is_unused(self) -> bool {
        self.0 == 0
    }
    const fn is_present(self) -> bool {
        (self.0 & (PageTableFlags::PRESENT.bits()) as usize) != 0
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("PageTableEntry");
        f.field("raw", &self.0)
            .field("pa", &self.pa())
            .field("flags", &self.flags())
            .finish()
    }
}

pub struct PageTable {
    pub root_pa: PhysAddr,
    tables: Vec<PhysFrame>,
}

impl PageTable {
    pub fn new() -> Self {
        let root_frame = PhysFrame::alloc_zero().unwrap();
        let p4 = table_of(root_frame.start_pa());
        p4[p4_index(VirtAddr(KERNEL_OFFSET))] = *KERNEL_PTE;
        p4[p4_index(VirtAddr(PHYS_OFFSET))] = *PHYS_PTE;
        Self {
            root_pa: root_frame.start_pa(),
            tables: vec![root_frame],
        }
    }

    pub fn map(&mut self, va: VirtAddr, pa: PhysAddr, flags: PageTableFlags) {
        let entry = self.get_entry_or_create(va).unwrap();
        if !entry.is_unused() {
            panic!("{:#x?} is mapped before mapping", va);
        }
        *entry = PageTableEntry::new_page(pa.align_down(), flags);
    }

    pub fn unmap(&mut self, va: VirtAddr) {
        let entry = self.get_entry(va).unwrap();
        if entry.is_unused() {
            panic!("{:#x?} is invalid before unmapping", va);
        }
        entry.0 = 0;
    }

    pub fn query(&self, va: VirtAddr) -> Option<(PhysAddr, PageTableFlags)> {
        let entry = self.get_entry(va)?;
        if entry.is_unused() {
            return None;
        }
        let off = va.page_offset();
        Some((PhysAddr(entry.pa().0 + off), entry.flags()))
    }

    pub fn map_area(&mut self, area: &mut MapArea) {
        assert!(area.start.0 + area.size < PHYS_OFFSET);
        let mut va = area.start.0;
        let end = va + area.size;
        while va < end {
            let pa = area.map(VirtAddr(va));
            self.map(VirtAddr(va), pa, area.flags);
            va += PAGE_SIZE;
        }
    }

    pub fn unmap_area(&mut self, area: &mut MapArea) {
        let mut va = area.start.0;
        let end = va + area.size;
        while va < end {
            area.unmap(VirtAddr(va));
            self.unmap(VirtAddr(va));
            va += PAGE_SIZE;
        }
    }

    #[allow(unused)]
    pub fn dump(&self, limit: usize) {
        println!("Root: {:x?}", self.root_pa);
        self.walk(
            table_of(self.root_pa),
            0,
            0,
            limit,
            &|level: usize, idx: usize, va: usize, entry: &PageTableEntry| {
                for _ in 0..level {
                    print!("  ");
                }
                println!("[{} - {:x}], {:08x?}: {:x?}", level, idx, va, entry);
            },
        );
    }
}

impl PageTable {
    fn alloc_table(&mut self) -> PhysAddr {
        let frame = PhysFrame::alloc_zero().unwrap();
        let pa = frame.start_pa();
        self.tables.push(frame);
        pa
    }

    fn get_entry(&self, va: VirtAddr) -> Option<&mut PageTableEntry> {
        let p4 = table_of(self.root_pa);
        let p4e = &mut p4[p4_index(va)];
        let p3 = next_table(p4e)?;
        let p3e = &mut p3[p3_index(va)];
        let p2 = next_table(p3e)?;
        let p2e = &mut p2[p2_index(va)];
        let p1 = next_table(p2e)?;
        let p1e = &mut p1[p1_index(va)];
        Some(p1e)
    }

    fn get_entry_or_create(&mut self, va: VirtAddr) -> Option<&mut PageTableEntry> {
        let p4 = table_of(self.root_pa);
        let p4e = &mut p4[p4_index(va)];
        let p3 = next_table_or_create(p4e, || self.alloc_table())?;
        let p3e = &mut p3[p3_index(va)];
        let p2 = next_table_or_create(p3e, || self.alloc_table())?;
        let p2e = &mut p2[p2_index(va)];
        let p1 = next_table_or_create(p2e, || self.alloc_table())?;
        let p1e = &mut p1[p1_index(va)];
        Some(p1e)
    }

    fn walk(
        &self,
        table: &[PageTableEntry],
        level: usize,
        start_va: usize,
        limit: usize,
        func: &impl Fn(usize, usize, usize, &PageTableEntry),
    ) {
        let mut n = 0;
        for (i, entry) in table.iter().enumerate() {
            let va = start_va + (i << (12 + (3 - level) * 9));
            if entry.is_present() {
                func(level, i, va, entry);
                if level < 3 {
                    let table_entry = next_table(entry).unwrap();
                    self.walk(table_entry, level + 1, va, limit, func);
                }
                n += 1;
                if n >= limit {
                    break;
                }
            }
        }
    }
}

const fn p4_index(va: VirtAddr) -> usize {
    (va.0 >> (12 + 27)) & (ENTRY_COUNT - 1)
}

const fn p3_index(va: VirtAddr) -> usize {
    (va.0 >> (12 + 18)) & (ENTRY_COUNT - 1)
}

const fn p2_index(va: VirtAddr) -> usize {
    (va.0 >> (12 + 9)) & (ENTRY_COUNT - 1)
}

const fn p1_index(va: VirtAddr) -> usize {
    (va.0 >> 12) & (ENTRY_COUNT - 1)
}

fn table_of<'a>(pa: PhysAddr) -> &'a mut [PageTableEntry] {
    let ptr = pa.kvaddr().as_ptr() as *mut _;
    unsafe { core::slice::from_raw_parts_mut(ptr, ENTRY_COUNT) }
}

fn next_table<'a>(entry: &PageTableEntry) -> Option<&'a mut [PageTableEntry]> {
    if entry.is_present() {
        Some(table_of(entry.pa()))
    } else {
        None
    }
}

fn next_table_or_create<'a>(
    entry: &mut PageTableEntry,
    mut alloc: impl FnMut() -> PhysAddr,
) -> Option<&'a mut [PageTableEntry]> {
    if entry.is_unused() {
        let pa = alloc();
        *entry = PageTableEntry::new_page(
            pa,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
        );
        Some(table_of(pa))
    } else {
        next_table(entry)
    }
}

pub fn init() {
    let cr3 = my_x86_64::get_cr3();
    let p4 = table_of(PhysAddr(cr3));
    *KERNEL_PTE.get() = p4[p4_index(VirtAddr(KERNEL_OFFSET))];
    *PHYS_PTE.get() = p4[p4_index(VirtAddr(PHYS_OFFSET))];
    // Cancel mapping in lowest addresses.
    p4[0].0 = 0;
}
