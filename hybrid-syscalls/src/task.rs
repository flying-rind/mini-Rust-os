//! 任务管理相关的系统调用
use super::*;
use future::executor;
use future::futures::{ThreadYield, WaitForProc, WaitForThread};
use log::info;
use mm::{MemoryArea, USER_STACK_BASE, USER_STACK_SIZE};
use trap::CURRENT_THREAD;

use alloc::string::ToString;
use alloc::sync::Arc;
use user_syscall::SysResult;
use x86_64::structures::paging::PageTableFlags;

/// Exit the current thread.
pub fn sys_exit(exit_code: usize) -> SysResult {
    // 退出当前进程
    let cur = current_thread();
    let tid = cur.tid();
    info!("Thread exit, tid: {}, code: {}", tid, exit_code);
    cur.exit();
    Ok(0)
}

/// 创建新进程
///
/// 并设置其主线程为就绪态
///
/// 若创建失败返回usize::MAX
pub fn sys_proc_create(name_ptr: usize, path_ptr: usize, args_ptr: usize) -> (usize, usize) {
    let name = unsafe { (*(name_ptr as *const &str)).to_string() };
    let path = unsafe { (*(path_ptr as *const &str)).to_string() };
    // 获取命令行参数从用户堆拷贝到内核堆
    let args: Option<Vec<String>> = if args_ptr != 0 {
        let args_ref: &Vec<String> = unsafe { &(*(args_ptr as *const Vec<String>)) };
        // [Debug]
        println!(
            "path_ptr = {:x}, args_ptr = {:x}, args[0] = {}",
            path_ptr, args_ptr, args_ref[0]
        );
        Some(args_ref.clone())
    } else {
        None
    };
    let new_process = Process::new(name, &path, args);
    if new_process.is_none() {
        return (usize::MAX, 0);
    }
    let new_process = new_process.unwrap();
    let new_process_id = new_process.pid();
    // 获取当前进程
    let current_process = CURRENT_THREAD.get().as_ref().unwrap().proc().unwrap();
    // 加入到父进程的子进程列表中
    current_process.add_child(new_process.clone());
    new_process.set_parent(Arc::downgrade(&current_process));
    // 设置进程就绪
    new_process.root_thread().resume();
    (new_process_id, 0)
}

/// 替换当前进程elf
///
/// 若失败返回usize::MAX
pub fn sys_exec(
    pathp: *const u8,
    argvp: *const *const u8,
    envp: *const *const u8,
) -> (usize, usize) {
    info!(
        "exec: pathp: {:?}, argvp: {:?}, envp: {:?}",
        pathp, argvp, envp
    );
    let path = unsafe { *(path_ptr as *const &str) };
    let path = path.to_string();
    // 获取命令行参数从用户堆拷贝到内核堆
    let args: Option<Vec<String>> = if args_ptr != 0 {
        let args_ref: &Vec<String> = unsafe { &(*(args_ptr as *const Vec<String>)) };
        Some(args_ref.clone())
    } else {
        None
    };
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    let current_proc = current_thread.proc().unwrap();
    (current_proc.exec(&path, args), 0)
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
    let flags =
        PageTableFlags::WRITABLE | PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
    // 分配用户栈
    let stack_area = MemoryArea::new(sp_base, USER_STACK_SIZE, flags, mm::MemAreaType::USERSTACK);
    // 插入到当前进程所在的地址空间中
    let current_memoryset = current_proc.memory_set();
    current_memoryset.insert_area(stack_area.clone());
    let new_thread = Thread::new(
        Arc::downgrade(&current_proc),
        tid,
        entry,
        sp_base + USER_STACK_SIZE,
        arg1,
        arg2,
        stack_area,
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
