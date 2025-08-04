//! 封装内核提供的系统调用给用户程序使用
#![no_std]

pub mod num;

use num_traits::FromPrimitive;
extern crate alloc;
pub use alloc::string::String;
use hal::SysError;

pub use custom::*;
pub use fs::*;
// pub use sync::*;
pub use print::*;
pub use task::*;

pub type SysResult = Result<usize, hal::SysError>;

pub mod custom;
pub mod debug;
pub mod fs;
pub mod print;
pub mod sync;
pub mod task;

/// 系统调用号
pub enum SyscallNum {
    // TASK
    Fork = 57,
    Vfork = 58,
    Execve = 59,
    Exit = 60,
    Wait = 61,
    // FS
    Write = 1,
    Read = 0,
    Close = 3,
    Dup = 292,
    Open = 2,
    // Custom
    TestCstr = 999,
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
        err => Err(SysError::from_isize(-err).unwrap()),
    }
}

pub fn sys_fork() -> SysResult {
    syscall(SyscallNum::Fork, [0x0; 6])
}

pub fn sys_vfork() -> SysResult {
    syscall(SyscallNum::Vfork, [0x0; 6])
}

pub fn sys_exec(path: *const u8, argvp: *const *const u8, envp: *const *const u8) -> SysResult {
    syscall(
        SyscallNum::Execve,
        [path as _, argvp as _, envp as _, 0, 0, 0],
    )
}

pub fn sys_exit(exit_code: usize) -> SysResult {
    syscall(SyscallNum::Exit, [exit_code, 0, 0, 0, 0, 0])
}

pub fn sys_wait(pid: usize, wstatus: *mut i32) -> SysResult {
    syscall(SyscallNum::Wait, [pid, wstatus as _, 0, 0, 0, 0])
}

pub fn sys_write(fd: usize, buf: *const u8, size: usize) -> SysResult {
    syscall(SyscallNum::Write, [fd, buf as _, size, 0, 0, 0])
}

pub fn sys_read(fd: usize, buf: *mut u8, size: usize) -> SysResult {
    syscall(SyscallNum::Read, [fd, buf as _, size, 0, 0, 0])
}

pub fn sys_close(fd: usize) -> SysResult {
    syscall(SyscallNum::Close, [fd, 0, 0, 0, 0, 0])
}

pub fn sys_open(path: *const u8, flags: usize, mode: usize) -> SysResult {
    syscall(SyscallNum::Open, [path as _, flags, mode, 0, 0, 0])
}

pub fn sys_dup(arg1: usize) -> SysResult {
    syscall(SyscallNum::Dup, [arg1, 0, 0, 0, 0, 0])
}

pub fn sys_test_cstr(argvp: *const *const u8) -> SysResult {
    syscall(SyscallNum::TestCstr, [argvp as _, 0, 0, 0, 0, 0])
}
