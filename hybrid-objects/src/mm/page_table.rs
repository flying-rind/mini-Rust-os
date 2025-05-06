//! 页表抽象
use super::*;
use crate::*;
use alloc::sync::Arc;
use memory_area::MemoryArea;
use physframe::PhysFrame;

use x86_64::structures::paging::PageTableFlags;

/// 全局变量，bootloader启动时会以固定偏移量`PHYS_OFFSET`
/// 映射整个物理内存,KERNEL_PTE为内核起始虚拟地址`KERNEL_OFFSET`
/// 在当前页4级页表中对应的页表项
static KERNEL_ELF_PTE: Cell<usize> = zero();

/// 内核栈在当前四级页表中的页表项
static KERNEL_STACK_PTE: Cell<usize> = zero();

/// 物理内存0在当前四级页表中对应的页表项
static PHYS_PTE: Cell<usize> = zero();

/// 1111..._0000_0000_0000
const PHYS_ADDR_MASK: usize = !(PAGE_SIZE - 1);

/// 页表
#[derive(Default)]
pub struct PageTable {
    /// 4级页表的起始物理地址
    root_pa: usize,
    /// 四级页表对应的四个物理页帧
    frames: Cell<Vec<PhysFrame>>,
}

impl PageTable {
    /// 创建一个新的页表
    ///
    /// 分配一个物理页帧
    pub fn new() -> Self {
        let root_frame = PhysFrame::alloc_zero().unwrap();

        // [Debug]
        // println!(
        //     "[Debugger] Created new page table, root_pa: 0x{:x}",
        //     root_frame.0
        // );

        // 设置共享地址空间
        let p4 = as_table(root_frame.0);
        // 共享内核代码
        p4[p4_index(KERNEL_OFFSET)] = *KERNEL_ELF_PTE;
        // 共享物理内存
        p4[p4_index(PHYS_OFFSET)] = *PHYS_PTE;
        // 共享内核栈
        p4[p4_index(KERNEL_STACK_BASE)] = *KERNEL_STACK_PTE;
        PageTable {
            root_pa: root_frame.0,
            frames: Cell::new(vec![root_frame]),
        }
    }

    /// 获取页表基地址
    pub fn paddr(&self) -> usize {
        self.root_pa
    }

    /// 获取一个虚拟地址的一级页表项，可能创建低级页表
    fn get_entry_or_create(&self, va: usize) -> &mut usize {
        let p4 = as_table(self.root_pa);
        let p4e = &mut p4[p4_index(va)];
        let p3 = next_table_or_create(p4e, self);
        let p3e = &mut p3[p3_index(va)];
        let p2 = next_table_or_create(p3e, self);
        let p2e = &mut p2[p2_index(va)];
        let p1 = next_table_or_create(p2e, self);
        let p1e = &mut p1[p1_index(va)];
        p1e
    }

    /// 将一对虚地址和物理地址映射写入页表
    pub fn map(&self, vaddr: usize, paddr: usize, _flags: PageTableFlags) {
        let entry = self.get_entry_or_create(vaddr);
        if !pte_is_empty(entry) {
            panic!("vaddr: 0x{:x} is mapped before", vaddr);
        }
        pte_set_table(entry, paddr);
    }

    /// 取消映射一个虚拟地址，清除对应的页表项
    pub fn unmap(&self, vaddr: usize) {
        let vaddr = align_down(vaddr);
        let entry = self.get_entry_or_create(vaddr);
        // 清空页表项
        *entry = 0;
    }

    /// 将一块虚存区域映射写入页表
    pub fn map_area(&mut self, area: Arc<MemoryArea>) {
        let mut vaddr = area.start_vaddr();
        let size = area.size();
        assert!(vaddr + size < PHYS_OFFSET);
        let end = vaddr + size;
        while vaddr < end {
            let paddr = area.map(vaddr);
            self.map(vaddr, paddr, area.flags());
            vaddr += PAGE_SIZE;
        }
    }

    /// 取消映射一块虚存区域，清除对应的页表项
    pub fn unmap_area(&mut self, area: Arc<MemoryArea>) {
        let mut vaddr = area.start_vaddr();
        let end = vaddr + area.size();
        while vaddr < end {
            area.unmap(vaddr);
            self.unmap(vaddr);
            vaddr += PAGE_SIZE;
        }
    }
}

/// 从虚拟地址获取四级页表索引，cr3中存四级页表首地址
///
/// 按这个索引取出的是三级页表首地址
const fn p4_index(va: usize) -> usize {
    (va >> (12 + 27)) & (ENTRY_COUNT - 1)
}

/// 从虚拟地址获取三级页表索引
const fn p3_index(va: usize) -> usize {
    (va >> (12 + 18)) & (ENTRY_COUNT - 1)
}

/// 从虚拟地址获取二级页表索引
const fn p2_index(va: usize) -> usize {
    (va >> (12 + 9)) & (ENTRY_COUNT - 1)
}

/// 从虚拟地址获取一级页表索引
const fn p1_index(va: usize) -> usize {
    (va >> 12) & (ENTRY_COUNT - 1)
}

/// 将给定物理地址paddr所在的页帧解析为页表项列表
fn as_table<'a>(pa: usize) -> &'a mut [usize] {
    let ptr = phys_to_virt(pa) as *mut usize;
    unsafe { core::slice::from_raw_parts_mut(ptr, ENTRY_COUNT) }
}

/// 由一个页表项获取下一级页表
///
/// 若页表项为空，则分配一个新的物理页帧创建新页表
fn next_table_or_create<'a>(entry: &mut usize, page_table: &PageTable) -> &'a mut [usize] {
    if pte_is_empty(entry) {
        // 分配一个物理页帧，作为新的一级页表的物理地址
        let frame = PhysFrame::alloc_zero().unwrap();
        pte_set_table(entry, frame.0);
        // 新页帧加入页表
        let paddr = frame.0;
        page_table.frames.get_mut().push(frame);
        as_table(paddr)
    } else {
        as_table((*entry) & PHYS_ADDR_MASK)
    }
}

/// 页表项是否为空
fn pte_is_empty(pte: &usize) -> bool {
    (*pte == 0) || (*pte == usize::MAX)
}

/// 物理地址写入页表项
pub fn pte_set_table(pte: &mut usize, paddr: usize) {
    *pte = paddr & PHYS_ADDR_MASK
        | (PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE)
            .bits() as usize;
}

/// 获取一个虚拟地址的四级页表项，可能创建低级页表
///
/// [Debug]用
fn get_entry(paddr: usize, va: usize, page_table: &PageTable) -> &'static mut usize {
    let p4 = as_table(paddr);
    let p4_ptr = p4.as_ptr() as usize;
    println!("[Debug pagetable] p4_ptr: 0x{:x}", p4_ptr);
    let p4_index = p4_index(va);
    println!("[Debug pagetable] p4_index: {}", p4_index);
    let p4e = &mut p4[p4_index];
    println!("[Debug pagetable] p4e: 0x{:x}", p4e);
    println!();

    let p3 = next_table_or_create(p4e, page_table);
    let p3_ptr = p3.as_ptr() as usize;
    println!("[Debug pagetable] p3_ptr: 0x{:x}", p3_ptr);
    let p3_index = p3_index(va);
    println!("[Debug pagetable] p3_index: {}", p3_index);
    let p3e = &mut p3[p3_index];
    println!("[Debug pagetable] p3e: 0x{:x}", p3e);
    println!();

    let p2 = next_table_or_create(p3e, page_table);
    let p2_ptr = p2.as_ptr() as usize;
    println!("[Debug pagetable] p2_ptr: 0x{:x}", p2_ptr);
    let p2_index = p2_index(va);
    println!("[Debug pagetable] p2_index: {}", p2_index);
    let p2e = &mut p2[p2_index];
    println!("[Debug pagetable] p2e: 0x{:x}", p2e);
    println!();

    let p1 = next_table_or_create(p2e, page_table);
    let p1_ptr = p1.as_ptr() as usize;
    println!("[Debug pagetable] p1_ptr: 0x{:x}", p1_ptr);
    let p1_index = p1_index(va);
    println!("[Debug pagetable] p1_indx: {}", p1_index);
    let p1e = &mut p1[p1_index];
    println!("[Debug pagetable] p1e: 0x{:x}", p1e);
    println!();

    p1e
}

/// 查询一个虚地址对应的物理地址
pub fn query(paddr: usize, vaddr: usize, page_table: &PageTable) -> usize {
    let entry = get_entry(paddr, vaddr, page_table);
    if pte_is_empty(entry) {
        panic!()
    }
    let offset = vaddr & (PAGE_SIZE - 1);
    return ((*entry) & PHYS_ADDR_MASK) + offset;
}

/// 页表管理初始化
pub(crate) fn init() {
    // 获取内核空间的四级页表
    let cr3 = my_x86_64::get_cr3();
    let p4 = as_table(cr3);
    *KERNEL_ELF_PTE.get_mut() = p4[p4_index(KERNEL_OFFSET)];
    *KERNEL_STACK_PTE.get_mut() = p4[p4_index(KERNEL_STACK_BASE)];
    *PHYS_PTE.get_mut() = p4[p4_index(PHYS_OFFSET)];
    // Cancel mapping in lowest addresses.
    p4[0] = 0;
}
