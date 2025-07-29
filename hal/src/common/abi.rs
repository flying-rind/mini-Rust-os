//! For musl abi?

use crate::alloc::string::String;
use alloc::collections::BTreeMap;
use alloc::slice;
use alloc::vec::Vec;
use core::ptr::null;
use xmas_elf::ElfFile;
use xmas_elf::program::Type;

/// 进程初始化时压入用户栈的信息
pub struct ProcInfo {
    pub args: Vec<String>,
    pub envs: Vec<String>,
    pub auxv: BTreeMap<u8, usize>,
}

impl ProcInfo {
    /// 将进程初始化信息压栈
    ///
    /// - high
    /// - argv0(Program name)
    /// - env0
    /// - env1
    /// - ...
    /// - argv0
    /// - argv1
    /// - null
    /// - ...
    /// - envp1
    /// - envp0
    /// - null
    /// - ...
    /// - argvp1
    /// - argvp0
    /// - ...
    /// - argc
    /// - low
    pub unsafe fn push_at(&self, stack_top: usize) -> usize {
        let mut writer = StackWriter { sp: stack_top };
        // program name
        writer.push_str(&self.args[0]);
        // env strings
        let envs: Vec<_> = self
            .envs
            .iter()
            .map(|arg| {
                writer.push_str(arg.as_str());
                writer.sp
            })
            .collect();
        // argv strings
        let argv: Vec<_> = self
            .args
            .iter()
            .map(|arg| {
                writer.push_str(arg.as_str());
                writer.sp
            })
            .collect();
        // auxiliary vector entries
        writer.push_slice(&[null::<u8>(), null::<u8>()]);
        for (&type_, &value) in self.auxv.iter() {
            writer.push_slice(&[type_ as usize, value]);
        }
        // envps
        writer.push_slice(&[null::<u8>]);
        writer.push_slice(envs.as_slice());
        // argvp
        writer.push_slice(&[null::<u8>]);
        writer.push_slice(argv.as_slice());
        // argc
        writer.push_slice(&[argv.len()]);
        writer.sp
    }
}

/// 辅助结构，用于压栈
struct StackWriter {
    sp: usize,
}

impl StackWriter {
    /// 切片压栈
    fn push_slice<T: Copy>(&mut self, vs: &[T]) {
        self.sp -= vs.len() * size_of::<T>();
        self.sp -= self.sp % align_of::<T>();
        unsafe { slice::from_raw_parts_mut(self.sp as *mut T, vs.len()) }.copy_from_slice(vs);
    }

    /// 字符串压栈
    fn push_str(&mut self, s: &str) {
        self.push_slice(&[b'\0']);
        self.push_slice(s.as_bytes());
    }
}

pub const AT_PHDR: u8 = 3;
pub const AT_PHENT: u8 = 4;
pub const AT_PHNUM: u8 = 5;
pub const AT_PAGESZ: u8 = 6;
pub const AT_BASE: u8 = 7;
pub const AT_ENTRY: u8 = 9;

/// Helper functions to process ELF file
///
/// Copied from rCore.
pub trait ElfExt {
    /// Get virtual address of PHDR section if it has.
    fn get_phdr_vaddr(&self) -> Option<u64>;
}

impl ElfExt for ElfFile<'_> {
    fn get_phdr_vaddr(&self) -> Option<u64> {
        if let Some(phdr) = self
            .program_iter()
            .find(|ph| ph.get_type() == Ok(Type::Phdr))
        {
            // if phdr exists in program header, use it
            Some(phdr.virtual_addr())
        } else if let Some(elf_addr) = self
            .program_iter()
            .find(|ph| ph.get_type() == Ok(Type::Load) && ph.offset() == 0)
        {
            // otherwise, check if elf is loaded from the beginning, then phdr can be inferred.
            Some(elf_addr.virtual_addr() + self.header.pt2.ph_offset())
        } else {
            warn!("elf: no phdr found, tls might not work");
            None
        }
    }
}
