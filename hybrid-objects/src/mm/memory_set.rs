//! 进程地址空间
use super::*;
use x86_64::registers::control::Cr3;
use x86_64::registers::control::Cr3Flags;
use x86_64::structures::paging::PageTableFlags;
use x86_64::structures::paging::PhysFrame;
use xmas_elf::{
    program::{SegmentData, Type},
    {header, ElfFile},
};

use super::memory_area::MemoryArea;
use super::PageTable;
use crate::Cell;
use alloc::{sync::Arc, vec::Vec};

/// 进程地址空间
#[derive(Default)]
pub struct MemorySet {
    /// 地址空间包含的虚存区域
    areas: Cell<Vec<Arc<MemoryArea>>>,
    /// 页表
    page_table: Arc<Cell<PageTable>>,
}

impl MemorySet {
    /// 新建地址空间
    pub fn new() -> Arc<MemorySet> {
        Arc::new(MemorySet {
            areas: Cell::new(Vec::new()),
            page_table: Arc::new(Cell::new(PageTable::new())),
        })
    }

    /// 插入一段虚存区域
    pub fn insert_area(&self, area: Arc<MemoryArea>) {
        self.areas.get_mut().push(area.clone());
        // 映射到页表中去
        self.page_table.get_mut().map_area(area);
    }

    /// 切换为当前地址空间，即修改cr3寄存器
    pub fn activate(&self) {
        let frame =
            PhysFrame::containing_address(x86_64::PhysAddr::new(self.page_table.paddr() as _));
        if Cr3::read().0 != frame {
            unsafe { Cr3::write(frame, Cr3Flags::empty()) };
        }
    }

    /// 清理地址空间中ELF类型的区域，取消映射
    pub fn clear_elf(&self) {
        let areas = self.areas.get_mut();
        areas.retain(|area| {
            if area.mtype() == MemAreaType::ELF {
                // 取消页表映射
                self.page_table.get_mut().unmap_area(area.clone());
                return false;
            }
            true
        });
    }

    /// 克隆一个地址空间时，克隆其中所有的虚存区域
    ///
    /// 用户栈不复制而是在进程复制时手动复制(因为一个地址空间有多个)
    pub fn clone_myself(&self) -> Arc<Self> {
        let ms = Self::new();
        for area in self.areas.get() {
            // 不复制用户栈，fork时手动复制
            if area.mtype() != MemAreaType::USERSTACK {
                ms.insert_area(area.clone_myself());
            }
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
        for area in self.areas.get() {
            self.page_table.get_mut().unmap_area(area.clone());
        }
        self.areas.clear();
        // println!("[Rust] MemorySet dropped now");
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
        elf.header.pt2.type_().as_type(),
        header::Type::Executable,
        "ELF is not an executable object"
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
        if flags.contains(PageTableFlags::WRITABLE) {
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
        let memory_area = MemoryArea::new(
            vaddr_start,
            vaddr_end - vaddr_start,
            flags,
            MemAreaType::ELF,
        );

        // 数据写入虚存区域中
        memory_area.write_data(offset, data);
        ms.insert_area(memory_area);
    }
}
