//! 任务管理相关的系统调用
use super::*;
use future::executor;
use future::futures::{ThreadYield, WaitForProc, WaitForThread};
use mm::{MemoryArea, USER_STACK_BASE, USER_STACK_SIZE};
use trap::CURRENT_THREAD;

use alloc::string::ToString;
use alloc::sync::Arc;
use x86_64::structures::paging::PageTableFlags;

/// 退出当前进程
pub fn sys_proc_exit(exit_code: usize) -> (usize, usize) {
    // 退出当前进程
    let cur = CURRENT_THREAD.get().as_ref().unwrap().clone();
    cur.proc().unwrap().exit();
    // println!(
    //     "[Kernel] proc `{}` exited with exit code `{}`",
    //     cur.proc().unwrap().name(),
    //     exit_code,
    // );
    (exit_code, 0)
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
pub fn sys_exec(path_ptr: usize, args_ptr: usize) -> (usize, usize) {
    let path = unsafe { *(path_ptr as *const &str) };
    let path = path.to_string();
    // println!(
    //     "in kernel sys_exec, path: {}, path_len:{}",
    //     path,
    //     path.len()
    // );
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

/// 当前线程等待一个进程结束
///
/// 若等待的进程不存在则返回255
pub fn sys_proc_wait(pid: usize) -> (usize, usize) {
    // 获取当前线程
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();

    let waited_process = match PROCESS_MAP.get().get(&pid) {
        Some(process) => process.clone(),
        None => {
            // println!("[Kernel] waited proc does not existed or already dropped");
            return (255, 0);
        }
    };
    current_thread.set_state(ThreadState::Waiting);
    executor::spawn(WaitForProc::new(current_thread, waited_process));
    (0, 0)
}

/// 当前线程放弃CPU
pub fn sys_yield() -> (usize, usize) {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    executor::spawn(ThreadYield::new(current_thread));
    return (0, 0);
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
pub fn sys_get_pid() -> (usize, usize) {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    (current_thread.proc().unwrap().pid(), 0)
}

/// 获取当前线程tid
pub fn sys_get_tid() -> (usize, usize) {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    (current_thread.tid(), 0)
}

/// 复制当前进程
pub fn sys_fork() -> (usize, usize) {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    let current_proc = current_thread.proc().unwrap();
    let child_proc = current_proc.fork();
    (child_proc.pid(), 0)
}
