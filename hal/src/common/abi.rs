//! For musl abi?

use crate::alloc::string::String;
use alloc::slice;
use alloc::vec::Vec;
use core::ptr::null;

/// 进程初始化时压入用户栈的信息
pub struct ProcInfo {
    pub args: Vec<String>,
    pub envs: Vec<String>,
    // auxv: what for?
}

impl ProcInfo {
    /// 将进程初始化信息压栈
    ///
    /// - high
    /// - env0
    /// - env1
    /// - ...
    /// - argv0
    /// - argv1
    /// - ...
    /// - null
    /// - envp0
    /// - envp1
    /// - ...
    /// - null
    /// - argvp0
    /// - argvp1
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
