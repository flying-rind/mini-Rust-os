//! 用于中断处理和系统调用
use crate::{my_x86_64::*, *};
use core::mem::size_of_val;

core::arch::global_asm!(include_str!("trap.S"));

mod handler;

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
/// 调用者保存寄存器
pub struct CallerRegs {
    pub rax: usize,
    pub rcx: usize,
    pub rdx: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub r8: usize,
    pub r9: usize,
    pub r10: usize,
    pub r11: usize,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
/// 被调用者保存寄存器
pub struct CalleeRegs {
    pub rsp: usize,
    pub rbx: usize,
    pub rbp: usize,
    pub r12: usize,
    pub r13: usize,
    pub r14: usize,
    pub r15: usize,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct SyscallFrame {
    pub caller: CallerRegs,
    pub callee: CalleeRegs,
}

const TSS_SIZE: usize = 104;

extern "C" {
    static TSS: [u8; TSS_SIZE];
    // 系统调用入口函数，定义在trap.s中
    fn syscall_entry();
    // 系统调用返回前调用，定义在trap.s中
    pub fn syscall_return(f: &SyscallFrame) -> !;
}

pub fn init() {
    // 初始化GDT
    static GDT: Cell<[usize; 7]> = Cell::new([
        0,
        0x00209800_00000000, // KCODE, EXECUTABLE | USER_SEGMENT | PRESENT | LONG_MODE
        0x00009200_00000000, // KDATA, DATA_WRITABLE | USER_SEGMENT | PRESENT
        0x0000F200_00000000, // UDATA, DATA_WRITABLE | USER_SEGMENT | USER_MODE | PRESENT
        0x0020F800_00000000, // UCODE, EXECUTABLE | USER_SEGMENT | USER_MODE | PRESENT | LONG_MODE
        0,
        0, // TSS, filled in runtime
    ]);
    let ptr = unsafe { TSS.as_ptr() as usize };
    let low = (1 << 47)
        | 0b1001 << 40
        | (TSS_SIZE - 1)
        | ((ptr & ((1 << 24) - 1)) << 16)
        | (((ptr >> 24) & ((1 << 8) - 1)) << 56);
    let high = ptr >> 32;
    GDT.get()[5] = low;
    GDT.get()[6] = high;
    lgdt(&DescriptorTablePointer {
        limit: size_of_val(&GDT) as u16 - 1,
        base: GDT.as_ptr() as _,
    });
    // unsafe {
    //     write_msr(KERNEL_GSBASE_MSR, TSS.as_ptr() as _);
    // }
    my_x86_64::set_cs((1 << 3) | my_x86_64::RING0);
    my_x86_64::set_ss((2 << 3) | my_x86_64::RING0);

    load_tss((5 << 3) | RING0);
    set_msr(EFER_MSR, get_msr(EFER_MSR) | 1); // enable system call extensions
    set_msr(STAR_MSR, (2 << 3 << 48) | (1 << 3 << 32));
    set_msr(LSTAR_MSR, syscall_entry as _);
    set_msr(SFMASK_MSR, 0x47700); // TF|DF|IF|IOPL|AC|NT
}
