//! 封装内核提供的系统调用给用户程序使用
#![no_std]
mod debug;
mod fs;
pub mod print;
mod sync;
mod task;

extern crate alloc;
extern crate bitflags;
extern crate num_derive;

use num_derive::FromPrimitive;

pub use debug::*;
pub use fs::*;
pub use sync::*;
pub use task::*;

/// 枚举系统调用
#[derive(FromPrimitive)]
pub enum SyscallNum {
    /// 打印信息
    DebugWrite,
    /// 测试用户到内核的数据传输
    DebugDataTransport,
    /// 测试是否能访问内核全局文件系统
    DebugOpen,
    /// 从串口读取一个字符
    SerialRead,
    /// 获取时间
    GetTime,

    /// 退出进程
    ProcExit,
    /// 创建进程
    ProcCreate,
    /// 等待进程
    ProcWait,
    /// 出让CPU
    Yield,
    /// 创建线程
    ThreadCreate,
    /// 当前线程退出
    ThreadExit,
    /// 等待线程结束
    ThreadJoin,
    /// 获取进程id
    GetPid,
    /// 获取线程id
    GetTid,
    /// 复制当前进程
    Fork,
    /// 替换当前进程elf
    Exec,

    /// 当前进程打开文件
    Open,
    /// 关闭进程打开的文件
    Close,
    /// 读文件
    Read,
    /// 写文件
    Write,
    /// 创建管道
    Pipe,
    /// 复制文件
    Dup,
    /// 列出可用的用户程序
    Ls,

    /// 创建互斥锁
    MutexCreate,
    /// 加锁
    MutexLock,
    /// 解锁
    MutexUnlock,
    /// 创建信号量
    SemCreate,
    /// 增加信号量资源
    SemUp,
    /// 减少信号量资源
    SemDown,
    /// 创建条件变量
    CondvarCreate,
    /// 阻塞条件变量
    CondvarWait,
    /// 唤醒条件变量
    CondvarSignal,
}

/// 用户态使用系统调用
fn syscall(id: SyscallNum, args: [usize; 6]) -> (usize, usize) {
    let mut ret0: usize;
    let mut ret1: usize;
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
            lateout("rdx") ret1,
        );
    }
    (ret0, ret1)
}

fn sys_debug_write(msg_ptr: usize) -> (usize, usize) {
    syscall(SyscallNum::DebugWrite, [msg_ptr, 0, 0, 0, 0, 0])
}

fn sys_serial_read(buf_ptr: usize) -> (usize, usize) {
    syscall(SyscallNum::SerialRead, [buf_ptr, 0, 0, 0, 0, 0])
}

fn sys_debug_data_transport(bufs_ptr: usize, ret_ptr: usize) -> (usize, usize) {
    syscall(
        SyscallNum::DebugDataTransport,
        [bufs_ptr, ret_ptr, 0, 0, 0, 0],
    )
}

fn sys_debug_open(name_ptr: usize) -> (usize, usize) {
    syscall(SyscallNum::DebugOpen, [name_ptr, 0, 0, 0, 0, 0])
}

fn sys_proc_exit(exit_code: usize) -> (usize, usize) {
    syscall(SyscallNum::ProcExit, [exit_code, 0, 0, 0, 0, 0])
}

fn sys_proc_wait(pid: usize) -> (usize, usize) {
    syscall(SyscallNum::ProcWait, [pid, 0, 0, 0, 0, 0])
}

fn sys_proc_create(name_ptr: usize, path_ptr: usize, args_ptr: usize) -> (usize, usize) {
    syscall(
        SyscallNum::ProcCreate,
        [name_ptr, path_ptr, args_ptr, 0, 0, 0],
    )
}

fn sys_yield() -> (usize, usize) {
    syscall(SyscallNum::Yield, [0, 0, 0, 0, 0, 0])
}

fn sys_open(path_ptr: usize, flags: usize) -> (usize, usize) {
    // 将fd指针传送到内核，内核转发到内核线程，最终服务完成后将结果写入fd,
    // 这样做是因为这个系统调用是异步的，不能直接使用寄存器返回fd的值
    let mut fd: usize = 0;
    let fd_ptr = &mut fd as *mut usize as usize;
    // 如果fs线程不存在或其他错误，则内核返回usize::MAX
    let (ret1, _) = syscall(SyscallNum::Open, [path_ptr, flags, fd_ptr, 0, 0, 0]);
    if ret1 == usize::MAX {
        return (usize::MAX, 0);
    }
    (fd, 0)
}

fn sys_read(fd: usize, buf_ptr: usize, buf_len: usize) -> (usize, usize) {
    let mut read_size: usize = 1;
    let result_ptr = &mut read_size as *mut usize as usize;
    let (ret1, _) = syscall(SyscallNum::Read, [fd, buf_ptr, buf_len, result_ptr, 0, 0]);
    if ret1 == usize::MAX {
        // 出错了
        return (usize::MAX, 0);
    } else if ret1 != 0 {
        // 是标准输入输出，同步返回了
        read_size = ret1;
    }
    // [Debug]
    // println!(
    //     "ret1 = {}, result_ptr: {:x}, read size: {}",
    //     ret1, result_ptr, read_size
    // );
    (read_size, 0)
}

fn sys_write(fd: usize, buf_ptr: usize, buf_len: usize) -> (usize, usize) {
    let mut write_size: usize = 0;
    let result_ptr = &mut write_size as *mut usize as usize;
    let (ret1, _) = syscall(SyscallNum::Write, [fd, buf_ptr, buf_len, result_ptr, 0, 0]);
    if ret1 == usize::MAX {
        // 出错
        return (usize::MAX, 0);
    } else if ret1 != 0 {
        // 标准输出，同步返回
        write_size = ret1;
    }
    (write_size, 0)
}

fn sys_close(fd: usize) -> (usize, usize) {
    let (ret1, _) = syscall(SyscallNum::Close, [fd, 0, 0, 0, 0, 0]);
    (ret1, 0)
}

fn sys_pipe() -> (usize, usize) {
    syscall(SyscallNum::Pipe, [0, 0, 0, 0, 0, 0])
}

fn sys_dup(fd: usize) -> (usize, usize) {
    syscall(SyscallNum::Dup, [fd, 0, 0, 0, 0, 0])
}

fn sys_ls() -> (usize, usize) {
    syscall(SyscallNum::Ls, [0, 0, 0, 0, 0, 0])
}

fn sys_mutex_create() -> (usize, usize) {
    syscall(SyscallNum::MutexCreate, [0, 0, 0, 0, 0, 0])
}

fn sys_mutex_lock(id: usize) -> (usize, usize) {
    syscall(SyscallNum::MutexLock, [id, 0, 0, 0, 0, 0])
}

fn sys_mutex_unlock(id: usize) -> (usize, usize) {
    syscall(SyscallNum::MutexUnlock, [id, 0, 0, 0, 0, 0])
}

fn sys_thread_create(entry: usize, arg1: usize, arg2: usize) -> (usize, usize) {
    syscall(SyscallNum::ThreadCreate, [entry, arg1, arg2, 0, 0, 0])
}

fn sys_thread_exit() -> (usize, usize) {
    syscall(SyscallNum::ThreadExit, [0, 0, 0, 0, 0, 0])
}

fn sys_thread_join(tid: usize) -> (usize, usize) {
    syscall(SyscallNum::ThreadJoin, [tid, 0, 0, 0, 0, 0])
}

fn sys_get_time() -> (usize, usize) {
    syscall(SyscallNum::GetTime, [0, 0, 0, 0, 0, 0])
}

fn sys_get_pid() -> (usize, usize) {
    syscall(SyscallNum::GetPid, [0, 0, 0, 0, 0, 0])
}

fn sys_get_tid() -> (usize, usize) {
    syscall(SyscallNum::GetTid, [0, 0, 0, 0, 0, 0])
}

fn sys_fork() -> (usize, usize) {
    syscall(SyscallNum::Fork, [0, 0, 0, 0, 0, 0])
}

fn sys_exec(path_ptr: usize, args_ptr: usize) -> (usize, usize) {
    syscall(SyscallNum::Exec, [path_ptr, args_ptr, 0, 0, 0, 0])
}

fn sys_sem_create(n: usize) -> (usize, usize) {
    syscall(SyscallNum::SemCreate, [n, 0, 0, 0, 0, 0])
}

fn sys_sem_up(sem_id: usize) -> (usize, usize) {
    syscall(SyscallNum::SemUp, [sem_id, 0, 0, 0, 0, 0])
}
fn sys_sem_down(sem_id: usize) -> (usize, usize) {
    syscall(SyscallNum::SemDown, [sem_id, 0, 0, 0, 0, 0])
}

fn sys_condvar_create() -> (usize, usize) {
    syscall(SyscallNum::CondvarCreate, [0, 0, 0, 0, 0, 0])
}

fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> (usize, usize) {
    syscall(SyscallNum::CondvarWait, [condvar_id, mutex_id, 0, 0, 0, 0])
}

fn sys_condvar_signal(condvar_id: usize) -> (usize, usize) {
    syscall(SyscallNum::CondvarSignal, [condvar_id, 0, 0, 0, 0, 0])
}
