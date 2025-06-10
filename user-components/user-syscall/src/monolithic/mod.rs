//! 宏内核系统调用
pub mod error;
pub mod fs;
pub mod task;

pub enum SyscallNum {
    Fork = 57,
    Vfork = 58,
    Execve = 59,
}

/// 用户态使用系统调用
fn syscall(id: SyscallNum, args: [usize; 6]) -> isize {
    let mut ret0: usize;
    let mut _ret1: usize;
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") id as usize,
            in("rdi") args[0],
            in("rsi") args[1],
            in("rdx") args[2],
            in("r10") args[3],
            in("r8") args[4],
            in("r9") args[5],
            out("rcx") _,
            out("r11") _,
            lateout("rax") ret0,
            lateout("rdx") _ret1,
        );
    }
    ret0 as _
}

pub fn sys_fork() -> isize {
    syscall(SyscallNum::Fork, [0x0; 6])
}

pub fn sys_vfork() -> isize {
    syscall(SyscallNum::Vfork, [0x0; 6])
}

pub fn sys_exec(path: *const u8, argv: *const *const u8, envp: *const *const u8) -> isize {
    syscall(
        SyscallNum::Execve,
        [path as usize, argv as usize, envp as usize, 0, 0, 0],
    )
}
