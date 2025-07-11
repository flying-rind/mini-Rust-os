//! 任务管理相关的系统调用
use super::*;
use alloc::sync::Arc;
use future::executor;
use future::futures::{ThreadYield, WaitForProc, WaitForThread};
use hal::{check_n_clone_cstr, check_n_clone_cstr_array};
use log::info;
use mm::{USER_STACK_BASE, USER_STACK_SIZE};
use trap::CURRENT_THREAD;
use user_syscall::SysResult;

/// Exit the current thread.
pub fn sys_exit(exit_code: usize) -> SysResult {
    // 退出当前进程
    let cur = current_thread();
    let tid = cur.tid();
    info!("Thread exit, tid: {}, code: {}", tid, exit_code);
    cur.exit();
    Ok(0)
}

/// 替换当前进程elf
///
/// 若失败返回usize::MAX
pub fn sys_exec(pathp: *const u8, argvp: *const *const u8, envp: *const *const u8) -> SysResult {
    info!(
        "exec: pathp: {:?}, argvp: {:?}, envp: {:?}",
        pathp, argvp, envp
    );
    let cur_proc = current_proc();
    let path = check_n_clone_cstr(pathp)?;
    let args = check_n_clone_cstr_array(argvp)?;
    let envs = check_n_clone_cstr_array(envp)?;
    let inode = cur_proc.lookup_inode(&path)?;
    Ok(cur_proc.exec(&inode, args, envs)?)
}

/// Wait 4 the process exit.
/// Return the PID. Currently no option argument yet so just wait for the process to exit.(FIXME, read zcore)
///
/// FIXME: Refactor to simplify this function.
/// See [wait(2)](https://man7.org/linux/man-pages/man2/waitpid.2.html)
pub fn sys_wait4(pid: usize) -> SysResult {
    // 获取当前线程
    let current_thread = current_thread();
    let waited_process = match PROCESS_MAP.get().get(&pid) {
        Some(process) => process.clone(),
        None => {
            info!("Wait complete, pid: {}", pid);
            return Ok(pid);
        }
    };
    current_thread.set_state(ThreadState::Waiting);
    executor::spawn(WaitForProc::new(current_thread, waited_process));
    Ok(pid)
}

/// 当前线程放弃CPU
pub fn sys_yield() -> SysResult {
    let current_thread = current_thread();
    current_thread.set_state(ThreadState::Waiting);
    executor::spawn(ThreadYield::new(current_thread));
    Ok(0)
}

/// 创建线程，返回tid
pub fn sys_thread_create(entry: usize, arg1: usize, arg2: usize) -> (usize, usize) {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    let current_proc = current_thread.proc().unwrap();
    let tid = current_proc.alloc_tid();
    // 每两个用户栈之间隔一段空间
    let sp_base = USER_STACK_BASE + tid * 2 * USER_STACK_SIZE;
    let new_thread = Thread::new(
        Arc::downgrade(&current_proc),
        tid,
        entry,
        sp_base + USER_STACK_SIZE,
    );
    new_thread.set_state(ThreadState::Runnable);
    current_proc.add_thread(new_thread);
    (tid, 0)
}

/// 退出当前线程
///
/// 设置为Exited状态等待调度器清理
pub fn sys_thread_exit() -> (usize, usize) {
    // 获取当前线程
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    current_thread.set_state(ThreadState::Exited);
    (0, 0)
}

/// 主线程等待tid线程
///
/// 若不是主线程调用，就报错并返回usize::MAX
pub fn sys_thread_join(tid: usize) -> (usize, usize) {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    let cur_tid = current_thread.tid();
    if cur_tid != 0 {
        println!(
            "[Kernel] Thread join failed, can only be called by root thread, current tid: {}",
            cur_tid
        );
        return (usize::MAX, 0);
    }
    // 获取tid对应的线程
    let waited_thread = current_thread.proc().unwrap().get_thread(tid);
    let waited_thread = match waited_thread {
        Some(waited_thread) => waited_thread,
        None => {
            // println!(
            //     "[Kernel] Thread join info: waited thread already exited!, tid: {}",
            //     tid
            // );
            return (usize::MAX, 0);
        }
    };
    // 创建等待协程
    current_thread.set_state(ThreadState::Waiting);
    executor::spawn(WaitForThread::new(current_thread, waited_thread));
    return (0, 0);
}

/// 获取当前进程PID
pub fn sys_get_pid() -> SysResult {
    let current_thread = current_thread();
    Ok(current_thread.proc().unwrap().pid())
}

/// 获取当前线程tid
pub fn sys_get_tid() -> SysResult {
    let current_thread = current_thread();
    Ok(current_thread.tid())
}

/// 复制当前进程
pub fn sys_fork() -> SysResult {
    let current_thread = current_thread();
    let current_proc = current_thread.proc().unwrap();
    let child_proc = current_proc.fork();
    Ok(child_proc.pid())
}
