//! 宏内核系统调用
pub mod error;
pub mod fs;
pub mod task;

pub use error::*;
use num_traits::FromPrimitive;
pub use task::*;
enum SyscallNum {
    Fork = 57,
    Vfork = 58,
    Execve = 59,
    Exit = 60,
}

/// 用户态使用系统调用
fn syscall(id: SyscallNum, args: [usize; 6]) -> SysResult {
    let mut ret0: isize;
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
    match ret0 {
        ret if ret >= 0 => Ok(ret as _),
        err => Err(SysError::from_isize(err).unwrap()),
    }
}

pub fn sys_fork() -> SysResult {
    syscall(SyscallNum::Fork, [0x0; 6])
}

pub fn sys_vfork() -> SysResult {
    syscall(SyscallNum::Vfork, [0x0; 6])
}

pub fn sys_exec(path: *const u8, argv: *const *const u8, envp: *const *const u8) -> SysResult {
    syscall(
        SyscallNum::Execve,
        [path as usize, argv as usize, envp as usize, 0, 0, 0],
    )
}

pub fn sys_exit(exit_code: usize) -> SysResult {
    syscall(SyscallNum::Exit, [exit_code, 0, 0, 0, 0, 0])
}
