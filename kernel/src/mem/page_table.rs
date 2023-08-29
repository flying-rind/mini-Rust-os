//! 虚拟内存映射

use x86_64::{instructions::tlb, structures::paging::PageTableFlags};

use super::{allocate_frame, phys_to_virt, PAGE_SIZE};

/// 每个页表中页表项的数量
const ENTRY_COUNT: usize = 512;

/// 页表项中代表物理地址的区域掩码
const PHYS_ADDR_MASK: usize = 0x000F_FFFF_FFFF_F000;

/// 主要实现页表映射相关功能
#[derive(Debug, Default)]
pub struct PageTableOldv {
    /// 页表的基地址
    paddr: usize,
}

impl PageTableOldv {
    /// 使用已有物理页帧作为基地址创建页表
    pub fn new(paddr: usize) -> Self {
        PageTableOldv { paddr }
    }

    /// 获取页表基地址
    pub fn paddr(&self) -> usize {
        self.paddr
    }

    /// 将paddr所在物理页帧映射至虚拟页page，并设置标记flags
    pub fn map(&self, vaddr: usize, paddr: usize, flags: PageTableFlags) {
        let vaddr = align_down(vaddr);
        let paddr = align_down(paddr);

        let entry = self.leaf_entry(vaddr);

        if !pte_is_empty(*entry) {
            let addr = pte_addr(*entry);
            if addr == paddr {
                pte_set_flags(entry, flags);
            } else {
                panic!("Already mapped");
            }
        } else {
            pte_set_addr(entry, paddr);
            pte_set_flags(entry, flags);
        }

        tlb::flush_all();
    }

    /// 返回给定虚地址的3级页表对应的页表项（期间可能需要创建中间页）
    pub fn leaf_entry(&self, vaddr: usize) -> &mut usize {
        let vaddr = align_down(vaddr);

        let l0 = self.as_table(self.paddr());
        let l0e = &mut l0[pte_l4_index(vaddr)];

        let l1 = self.next_table(l0e);
        let l1e = &mut l1[pte_l3_index(vaddr)];

        let l2 = self.next_table(l1e);
        let l2e = &mut l2[pte_l2_index(vaddr)];

        let l3 = self.next_table(l2e);
        let l3e = &mut l3[pte_l1_index(vaddr)];
        l3e
    }

    /// 给定页表项，获取此页表项对应的下一级页表
    pub fn next_table(&self, entry: &mut usize) -> &mut [usize] {
        if pte_is_empty(*entry) {
            let paddr = allocate_frame().expect("Failed to allocate frame");
            unsafe { core::ptr::write_bytes((phys_to_virt(paddr)) as *mut u8, 0, PAGE_SIZE) };
            pte_set_addr(entry, paddr);

            // 准备映射时的标识
            let flags = PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE;
            // let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            pte_set_flags(entry, flags);

            self.as_table(paddr)
        } else {
            if !pte_is_valid(*entry) {
                panic!("Not mapped");
            } else {
                self.as_table(pte_addr(*entry))
            }
        }
    }

    /// 给定物理地址paddr，将此地址对应的页帧解析为页表项列表
    pub fn as_table<'a>(&self, paddr: usize) -> &'a mut [usize] {
        let ptr = (phys_to_virt(paddr)) as *mut usize;
        unsafe { core::slice::from_raw_parts_mut(ptr, ENTRY_COUNT) }
    }
}

/// 向下对齐
fn align_down(addr: usize) -> usize {
    addr & !(PAGE_SIZE - 1)
}

/// 获取4级页表目录
fn pte_l4_index(pte: usize) -> usize {
    (pte >> (12 + 27)) & (ENTRY_COUNT - 1)
}

/// 获取3级页表目录
fn pte_l3_index(pte: usize) -> usize {
    (pte >> (12 + 18)) & (ENTRY_COUNT - 1)
}

/// 获取2级页表目录
fn pte_l2_index(pte: usize) -> usize {
    (pte >> (12 + 9)) & (ENTRY_COUNT - 1)
}

/// 获取1级页表目录
fn pte_l1_index(pte: usize) -> usize {
    (pte >> 12) & (ENTRY_COUNT - 1)
}

/// 返回页表项包含的物理地址
pub fn pte_addr(pte: usize) -> usize {
    pte & PHYS_ADDR_MASK
}

/// 页表项是否为空
pub fn pte_is_empty(pte: usize) -> bool {
    pte == 0
}

/// 页表项是否有效
pub fn pte_is_valid(pte: usize) -> bool {
    (pte & 1) != 0
}

/// 设置页表项包含的物理地址
pub fn pte_set_addr(pte: &mut usize, paddr: usize) {
    *pte = paddr & PHYS_ADDR_MASK;
}

/// 设置页表项包含的标识
pub fn pte_set_flags(pte: &mut usize, flags: PageTableFlags) {
    *pte = (*pte & PHYS_ADDR_MASK) | flags.bits() as usize;
    *pte = (*pte) & 0x7fff_ffff_ffff_ffff;
}

// 拿来主义

use super::*;
use crate::mem::memory_set::MapArea;
use crate::my_x86_64::get_cr3;
use crate::*;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

static KERNEL_PTE: Cell<PageTableEntry> = zero();
static PHYS_PTE: Cell<PageTableEntry> = zero();

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    const PHYS_ADDR_MASK: usize = !(PAGE_SIZE - 1);

    pub const fn new_page(pa: PhysAddr, flags: PageTableFlags) -> Self {
        Self((pa.0 & Self::PHYS_ADDR_MASK) | flags.bits() as usize)
    }
    const fn pa(self) -> PhysAddr {
        PhysAddr(self.0 as usize & Self::PHYS_ADDR_MASK)
    }
    const fn flags(self) -> PageTableFlags {
        // PTFlags::from_bits_truncate(self.0)
        PageTableFlags::from_bits_truncate(self.0 as u64)
    }
    const fn is_unused(self) -> bool {
        self.0 == 0
    }
    const fn is_present(self) -> bool {
        (self.0 & PageTableFlags::PRESENT.bits() as usize) != 0
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

fn to_user(s: PageTableEntry) -> PageTableEntry {
    PageTableEntry(s.0 | PageTableFlags::USER_ACCESSIBLE.bits() as usize)
}

impl PageTable {
    pub fn new() -> Self {
        let root_frame = PhysFrame::alloc_zero().unwrap();
        let p4 = table_of(root_frame.start_pa());
        // p4[p4_index(VirtAddr(KERNEL_OFFSET))] = *KERNEL_PTE;
        // p4[p4_index(VirtAddr(PHYS_OFFSET))] = *PHYS_PTE;
        let current_pte = table_of(PhysAddr(get_cr3()));
        for (index, entry) in current_pte.iter().enumerate() {
            p4[index] = *entry;
        }
        // p4[1] = to_user(current_pte[1]);
        // // p4[1] = current_pte[1];
        // p4[2] = to_user(current_pte[2]);
        // // p4[2] = current_pte[2];
        // p4[3] = to_user(current_pte[3]);
        // // p4[3] = current_pte[3];
        // p4[4] = to_user(current_pte[4]);
        // // p4[4] = current_pte[4];
        // p4[511] = to_user(current_pte[511]);
        // // p4[511] = current_pte[511];
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
        let entry = get_entry(self.root_pa, va).unwrap();
        if entry.is_unused() {
            panic!("{:#x?} is invalid before unmapping", va);
        }
        entry.0 = 0;
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
}

impl PageTable {
    fn alloc_table(&mut self) -> PhysAddr {
        let frame = PhysFrame::alloc_zero().unwrap();
        let pa = frame.start_pa();
        self.tables.push(frame);
        pa
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

pub fn query(root_pa: PhysAddr, va: VirtAddr) -> Option<(PhysAddr, PageTableFlags)> {
    let entry = get_entry(root_pa, va)?;
    if entry.is_unused() {
        return None;
    }
    let off = va.page_offset();
    Some((PhysAddr(entry.pa().0 + off), entry.flags()))
}

fn get_entry(root_pa: PhysAddr, va: VirtAddr) -> Option<&'static mut PageTableEntry> {
    let p4 = table_of(root_pa);
    let p4e = &mut p4[p4_index(va)];
    let p3 = next_table(p4e)?;
    let p3e = &mut p3[p3_index(va)];
    let p2 = next_table(p3e)?;
    let p2e = &mut p2[p2_index(va)];
    let p1 = next_table(p2e)?;
    let p1e = &mut p1[p1_index(va)];
    Some(p1e)
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

pub(crate) fn init() {
    let cr3 = my_x86_64::get_cr3();
    let p4 = table_of(PhysAddr(cr3));
    *KERNEL_PTE.get() = p4[p4_index(VirtAddr(KERNEL_OFFSET))];
    *PHYS_PTE.get() = p4[p4_index(VirtAddr(PHYS_OFFSET))];
    // Cancel mapping in lowest addresses.
    p4[0].0 = 0;
}
