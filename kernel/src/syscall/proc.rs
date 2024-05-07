use super::*;
use crate::{mm::*, task::*, *};

/// 当前线程主动退出
pub fn sys_exit(exit_code: i32) -> ! {
    task::current().exit(exit_code)
}

/// 当前线程主动放弃CPU
pub fn sys_yield() -> isize {
    task::current_yield();
    0
}

/// 获取当前线程所在进程的pid
pub fn sys_getpid() -> isize {
    task::current().proc.pid as _
}

/// 复制当前线程所在进程,返回pid
pub fn sys_fork() -> isize {
    task::current().proc.fork().pid as _
}

/// 当前线程所在进程入口设置为elf文件入口
pub fn sys_exec(path: *const u8, args: *const *const u8) -> isize {
    let path = try_!(read_cstr(path), EFAULT);
    let args = try_!(read_cstr_array(args), EFAULT);
    task::current().proc.exec(&path, args)
}

/// 回收子进程
/// 没有找到符合pid的子线程，返回-1
/// 找到子线程，但尚不能回收，返回-2
pub fn sys_waitpid(pid: isize, exit_code_p: *mut u32) -> isize {
    let (pid, exit_code) = task::current().proc.waitpid(pid);
    if pid >= 0 && !exit_code_p.is_null() {
        exit_code_p.write_user(exit_code as _);
    }
    pid
}

/// 创建一个线程，若需要则为其分配栈空间
///
///返回线程tid
pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
    let t = current();
    let (t1, need_stack) = Thread::new(t.proc, user_task_entry, 0);
    let stack = USTACK_TOP - t1.tid * USTACK_SIZE;
    // 为线程分配栈空间
    if need_stack {
        t.proc.vm.as_mut().unwrap().insert(MapArea::new(
            VirtAddr(stack - USTACK_SIZE),
            USTACK_SIZE,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
        ));
    }
    let f = t1.syscall_frame();
    f.caller.rcx = entry;
    f.caller.r11 = my_x86_64::RFLAGS_IF;
    f.callee.rsp = stack;
    f.caller.rdi = arg;
    t1.tid as _
}

/// 获取线程tid
pub fn sys_gettid() -> isize {
    current().tid as _
}

/// 等待一个线程结束
///
/// 若回收自己，返回-1
///
/// 若未能回收符合的线程，返回-2
pub fn sys_waittid(tid: usize) -> isize {
    let t = current();
    // 线程不能回收自己
    if t.tid == tid {
        return -1;
    }
    let t1 = try_!(t.proc.threads.get_mut(tid), -1);
    if t1.state == ThreadState::Zombie {
        t1.state = ThreadState::Waited;
        t1.exit_code as _
    } else {
        -2
    }
}
