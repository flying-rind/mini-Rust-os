//! 虚拟内存区域

use alloc::sync::Arc;
use hashbrown::{hash_map::Entry, HashMap};
use x86_64::structures::paging::PageTableFlags;

use super::physframe::PhysFrame;
use super::*;
use crate::mm::{align_down, is_aligned, phys_to_virt, PAGE_SIZE};
use crate::Cell;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::size_of;

/// 虚存区域的类型
#[derive(PartialEq, Eq, Clone)]
pub enum MemAreaType {
    /// ELF
    ELF,
    /// 用户栈
    USERSTACK,
}

/// 虚存区域
pub struct MemoryArea {
    /// 起始虚地址
    start_vaddr: usize,
    /// 虚拟内存区域长度
    size: usize,
    /// 映射标识
    flags: PageTableFlags,
    /// 映射关系
    mapper: Cell<HashMap<usize, PhysFrame>>,
    /// 类型
    mtype: MemAreaType,
}

impl MemoryArea {
    /// 新建一块虚存区域
    pub fn new(
        start_vaddr: usize,
        size: usize,
        flags: PageTableFlags,
        mtype: MemAreaType,
    ) -> Arc<Self> {
        assert!(is_aligned(start_vaddr) && is_aligned(size));
        Arc::new(MemoryArea {
            start_vaddr,
            size,
            flags,
            mapper: Cell::new(HashMap::new()),
            mtype,
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

    /// 获取起始虚地址
    pub fn start_vaddr(&self) -> usize {
        self.start_vaddr
    }

    /// 获得区域类型
    pub fn mtype(&self) -> MemAreaType {
        self.mtype.clone()
    }

    /// 获取虚存区域长度
    pub fn size(&self) -> usize {
        self.size
    }

    /// 获取映射标识
    pub fn flags(&self) -> PageTableFlags {
        self.flags
    }

    /// 在虚存区域的指定偏移处写入数据
    pub fn write_data(&self, offset: usize, data: &[u8]) {
        assert!(offset + data.len() < self.size);
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
            mtype: self.mtype.clone(),
        })
    }
}

/// 将命令行参数压入线程的用户栈中，返回（top, argc，argv）
///
/// 栈的情况：
///
/// ----high
///
/// \0
///
/// ptr2
///
/// ptr1    <--argv
///
/// str1
///
/// str2
///
/// ----low
pub fn push_to_stack(_stack: Arc<MemoryArea>, args: Option<Vec<String>>) -> (usize, usize, usize) {
    let args = args.unwrap();
    let mut top =
        (USER_STACK_BASE + USER_STACK_SIZE - (args.len() + 1) * size_of::<usize>()) as *mut u8;
    let argv = top as *mut usize;
    unsafe {
        for (i, arg) in args.iter().enumerate() {
            top = top.sub(arg.len() + 1);
            core::ptr::copy_nonoverlapping(arg.as_ptr(), top, arg.len());
            *(top.add(arg.len())) = 0; // '\0'
            *argv.add(i) = top as _;
        }
        // argv[argc] = NULL
        *argv.add(args.len()) = 0;
    }
    return (top as usize & !0xF, args.len(), argv as usize);
}
