//! 系统调用模块
mod debug;
mod fs;
mod sync;
mod task;

use crate::*;
use debug::*;
use fs::*;
use sync::*;
use task::*;
use user_syscall::SyscallNum::*;

/// 系统调用总控函数
pub fn syscall(syscall_id: usize, args: [usize; 6]) -> (usize, usize) {
    let syscall_id = num::FromPrimitive::from_usize(syscall_id).unwrap();
    let ret = match syscall_id {
        // 调试用
        DebugWrite => sys_debug_write(args[0]),
        DebugDataTransport => sys_debug_data_transport(args[0], args[1]),
        DebugOpen => sys_debug_open(args[0]),
        SerialRead => sys_serial_read(args[0]),
        GetTime => (*pic::TICKS as _, 0),

        // 任务相关
        ProcExit => sys_proc_exit(args[0]),
        ProcCreate => sys_proc_create(args[0], args[1], args[2]),
        ProcWait => sys_proc_wait(args[0]),
        Yield => sys_yield(),
        ThreadCreate => sys_thread_create(args[0], args[1], args[2]),
        ThreadExit => sys_thread_exit(),
        ThreadJoin => sys_thread_join(args[0]),
        GetPid => sys_get_pid(),
        GetTid => sys_get_tid(),
        Fork => sys_fork(),
        Exec => sys_exec(args[0], args[1]),

        // 文件相关
        Open => sys_open(args[0], args[1], args[2]),
        Close => sys_close(args[0]),
        Read => sys_read(args[0], args[1], args[2], args[3]),
        Write => sys_write(args[0], args[1], args[2], args[3]),
        Pipe => sys_pipe(),
        Dup => sys_dup(args[0]),
        Ls => sys_ls(),

        // 同步互斥
        MutexCreate => sys_mutex_create(),
        MutexLock => sys_mutex_lock(args[0]),
        MutexUnlock => sys_mutex_unlock(args[0]),
        SemCreate => sys_sem_create(args[0]),
        SemUp => sys_sem_up(args[0]),
        SemDown => sys_sem_down(args[0]),
        CondvarCreate => sys_condvar_create(),
        CondvarWait => sys_condvar_wait(args[0], args[1]),
        CondvarSignal => sys_condvar_signal(args[0]),
    };
    ret
}
