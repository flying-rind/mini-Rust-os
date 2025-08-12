//! 进程地址空间
use super::*;
use alloc::collections::btree_map::BTreeMap;
use alloc::vec::Vec;
use core::fmt::Debug;
use x86_64::registers::control::Cr3;
use x86_64::registers::control::Cr3Flags;
use x86_64::structures::paging::PageTableFlags;
use x86_64::structures::paging::PhysFrame;
use xmas_elf::{
    program::{SegmentData, Type},
    {ElfFile, header},
};

use super::PageTable;
use super::memory_area::MemoryArea;
use crate::Cell;
use alloc::sync::Arc;

/// 进程地址空间
#[derive(Default)]
pub struct MemorySet {
    /// 地址空间包含的虚存区域
    areas: Cell<BTreeMap<usize, Arc<MemoryArea>>>,
    /// 页表
    page_table: Arc<Cell<PageTable>>,
}

impl MemorySet {
    /// 新建地址空间
    pub fn new() -> Arc<MemorySet> {
        Arc::new(MemorySet {
            areas: Cell::new(BTreeMap::new()),
            page_table: Arc::new(Cell::new(PageTable::new())),
        })
    }

    /// 插入一段虚存区域
    pub fn insert_area(&self, area: Arc<MemoryArea>) {
        info!(
            "insert area, start address: {:#x}, size: {:#x}",
            area.start_vaddr(),
            area.size()
        );
        self.areas
            .get_mut()
            .insert(area.start_vaddr(), area.clone());
        // 映射到页表中去
        self.page_table.get_mut().map_area(area);
    }

    /// Remove a mem area.
    pub fn remove_area(&self, addr: usize) {
        if let Some((addr, area)) = self.areas.get().get_key_value(&addr) {
            info!(
                "remove area, start address: {:#x}, size: {:#x}",
                addr,
                area.size()
            );
            self.areas.get_mut().remove(&addr);
            self.page_table.get_mut().unmap_area(area.clone());
        }
    }

    /// Remove the area `[start_addr, end_addr]`.
    /// Split existed ones when necessary.
    pub fn remove_with_split(&self, start_addr: usize, end_addr: usize) {
        assert!(start_addr <= end_addr, "invalid memory area");
        let mut i = 0;
        let areas: Vec<_> = self.areas.values().cloned().collect();
        while i < areas.len() {
            if areas[i].is_overlap_with(start_addr, end_addr) {
                let area_start = areas[i].start_vaddr();
                let area_size = areas[i].size();
                let area_end = area_start + area_size;
                let flags = areas[i].flags();
                if area_start >= start_addr && area_end <= end_addr {
                    self.remove_area(area_start);
                    // i = i.wrapping_sub(1);
                } else if area_start >= start_addr && area_start < end_addr {
                    self.remove_area(area_start);
                    let new_area = MemoryArea::new(end_addr, area_end - end_addr, flags);
                    self.insert_area(new_area);
                } else if area_end > start_addr && area_end <= end_addr {
                    self.remove_area(area_start);
                    let new_area = MemoryArea::new(area_start, start_addr - area_start, flags);
                    self.insert_area(new_area);
                } else {
                    self.remove_area(area_start);
                    let new_area_left = MemoryArea::new(area_start, start_addr - area_start, flags);
                    let new_area_right = MemoryArea::new(end_addr, area_end - end_addr, flags);
                    self.insert_area(new_area_left);
                    self.insert_area(new_area_right);
                }
            }
            i = i.wrapping_add(1);
        }
    }

    /// Find a free area with hint address `address hint` and length `len`.
    /// Return the start addr of found free area.
    /// Used for mmap.
    pub fn find_free_area(&self, addr_hint: usize, len: usize) -> usize {
        core::iter::once(addr_hint)
            .chain(
                self.areas
                    .values()
                    .cloned()
                    .map(|area| area.start_vaddr() + area.size()),
            )
            .map(|addr| (addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1))
            .find(|&addr| self.test_free_area(addr, addr + len))
            .expect("failed to find free area!")
    }

    /// Test if [`start_addr`, `end_addr`] is a free area.
    fn test_free_area(&self, start_addr: usize, end_addr: usize) -> bool {
        self.areas
            .values()
            .cloned()
            .find(|area| area.is_overlap_with(start_addr, end_addr))
            .is_none()
    }

    /// Get areas.
    pub fn areas(&self) -> impl Iterator<Item = Arc<MemoryArea>> {
        self.areas.get().values().cloned()
    }

    /// 切换为当前地址空间，即修改cr3寄存器
    pub fn activate(&self) {
        let frame =
            PhysFrame::containing_address(x86_64::PhysAddr::new(self.page_table.paddr() as _));
        if Cr3::read().0 != frame {
            unsafe { Cr3::write(frame, Cr3Flags::empty()) };
        }
    }

    /// 克隆一个地址空间时，克隆其中所有的虚存区域
    pub fn clone_myself(&self) -> Arc<Self> {
        let ms = Self::new();
        for (_addr, area) in self.areas.get() {
            ms.insert_area(area.clone_myself());
        }
        ms
    }

    /// 获取页表
    pub fn page_table(&self) -> Arc<Cell<PageTable>> {
        self.page_table.clone()
    }
}

impl Drop for MemorySet {
    /// 析构时取消映射所有虚存区域
    fn drop(&mut self) {
        for (_addr, area) in self.areas.get() {
            self.page_table.get_mut().unmap_area(area.clone());
        }
        self.areas.clear();
        // println!("[Rust] MemorySet dropped now");
    }
}

impl Debug for MemorySet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let areas = self.areas.get();
        for area in areas {
            write!(f, "area: {:?}\n", area)?;
        }
        Ok(())
    }
}

/// 解析elf文件，为其中的每个Load段创建虚拟内存块
pub fn load_app(ms: Arc<MemorySet>, elf: &ElfFile) {
    assert_eq!(
        elf.header.pt1.class(),
        header::Class::SixtyFour,
        "64-bit ELF required"
    );
    assert_eq!(
        elf.header.pt2.machine().as_machine(),
        header::Machine::X86_64,
        "invalid ELF arch"
    );
    for ph in elf.program_iter() {
        if ph.get_type() != Ok(Type::Load) {
            continue;
        }
        // 准备映射标志
        let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
        if ph.flags().is_write() {
            flags |= PageTableFlags::WRITABLE;
        }
        let offset = ph.virtual_addr() as usize & (PAGE_SIZE - 1);
        let vaddr_start = align_down(ph.virtual_addr() as usize);
        let vaddr_end = align_up(ph.virtual_addr() as usize + ph.mem_size() as usize);
        // 读取ELF段数据并写入物理页帧组
        let data = match ph.get_data(&elf).unwrap() {
            SegmentData::Undefined(data) => data,
            _ => panic!("failed to get ELF segment data"),
        };
        let memory_area = MemoryArea::new(vaddr_start, vaddr_end - vaddr_start, flags);

        // 数据写入虚存区域中
        memory_area.write_data(offset, data);
        ms.insert_area(memory_area);
    }
}
