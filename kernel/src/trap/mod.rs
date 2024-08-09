//! 中断处理函数和相关数据结构
mod handler;

pub use self::handler::*;
pub use crate::task::*;

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct CalleeRegs {
    pub rsp: usize,
    pub rbx: usize,
    pub rbp: usize,
    pub r12: usize,
    pub r13: usize,
    pub r14: usize,
    pub r15: usize,
}

/// 使用trapframe库初始化gdt，idt和中断向量
pub fn init() {
    unsafe {
        trapframe::init();
    }
}
