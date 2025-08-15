//! 虚拟内存区域

use alloc::sync::Arc;
use hashbrown::{HashMap, hash_map::Entry};
use x86_64::structures::paging::PageTableFlags;

use super::physframe::PhysFrame;
use crate::Cell;
use crate::mm::{PAGE_SIZE, align_down, is_aligned, phys_to_virt};
use core::fmt::Debug;

/// 虚存区域
pub struct MemoryArea {
    /// 起始虚地址, Must be page aligned.
    start_vaddr: usize,
    /// 虚拟内存区域长度, must be page aligned.
    size: usize,
    /// 映射标识
    flags: PageTableFlags,
    /// 映射关系
    mapper: Cell<HashMap<usize, PhysFrame>>,
}

impl MemoryArea {
    /// 新建一块虚存区域
    pub fn new(start_vaddr: usize, size: usize, flags: PageTableFlags) -> Arc<Self> {
        assert!(is_aligned(start_vaddr) && is_aligned(size));
        Arc::new(MemoryArea {
            start_vaddr,
            size,
            flags,
            mapper: Cell::new(HashMap::new()),
        })
    }

    /// 获取一个虚地址映射的物理页帧，若没有则分配一个页帧
    pub fn map(&self, vaddr: usize) -> usize {
        assert!(is_aligned(vaddr));
        match self.mapper.get_mut().entry(vaddr) {
            Entry::Occupied(e) => e.get().0,
            Entry::Vacant(e) => e.insert(PhysFrame::alloc_zero().unwrap()).start_paddr(),
        }
    }

    /// 取消映射一个虚地址
    pub fn unmap(&self, vaddr: usize) {
        self.mapper.get_mut().remove(&vaddr);
    }

    /// Test if the area is overlap with area [`start_adrr`, `end_addr`].
    pub fn is_overlap_with(&self, start_addr: usize, end_addr: usize) -> bool {
        let p0 = self.start_vaddr / PAGE_SIZE;
        let p1 = (self.start_vaddr + self.size - 1) / PAGE_SIZE + 1;
        let p2 = start_addr / PAGE_SIZE;
        let p3 = (end_addr - 1) / PAGE_SIZE + 1;
        !(p1 <= p2 || p3 <= p0)
    }

    /// 获取起始虚地址
    #[inline(always)]
    pub fn start_vaddr(&self) -> usize {
        self.start_vaddr
    }

    /// 获取虚存区域长度
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.size
    }

    /// 获取映射标识
    #[inline(always)]
    pub fn flags(&self) -> PageTableFlags {
        self.flags
    }

    /// 在虚存区域的指定偏移处写入数据
    pub fn write_data(&self, offset: usize, data: &[u8]) {
        assert!(offset + data.len() <= self.size);
        let mut start = offset;
        let mut remain = data.len();
        let mut processed = 0;
        while remain > 0 {
            let start_align = align_down(start);
            let page_offset = start - start_align;
            // 本次复制的长度
            let n = (PAGE_SIZE - page_offset).min(remain);
            // 获取（可能创建页帧）物理地址
            let paddr = self.map(self.start_vaddr + start_align);
            // 写入
            unsafe {
                core::slice::from_raw_parts_mut(
                    (phys_to_virt(paddr) as *mut u8).add(page_offset),
                    n,
                )
                .copy_from_slice(&data[processed..processed + n])
            }
            start += n;
            remain -= n;
            processed += n;
        }
    }

    /// 克隆这个虚存区域
    pub fn clone_myself(&self) -> Arc<MemoryArea> {
        let mut mapper = Cell::new(HashMap::new());
        // 为每个虚地址分配新的物理地址，且复制原数据
        for (&vaddr, frame) in self.mapper.get() {
            let new_frame = PhysFrame::alloc().unwrap();
            new_frame.as_slice().copy_from_slice(frame.as_slice());
            mapper.insert(vaddr, new_frame);
        }
        Arc::new(Self {
            start_vaddr: self.start_vaddr,
            size: self.size,
            flags: self.flags,
            mapper,
        })
    }
}

impl Debug for MemoryArea {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Memory area, start vaddr: 0x{:x}, size: 0x{:x}, flags: {:#?}",
            self.start_vaddr, self.size, self.flags
        )
    }
}
