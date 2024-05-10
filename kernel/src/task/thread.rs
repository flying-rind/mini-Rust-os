//! 线程管理

use super::*;
use crate::{trap::*, *};

core::arch::global_asm!(include_str!("switch.S"));

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct Context {
    pub regs: CalleeRegs,
    pub rip: usize,
}

extern "C" {
    pub fn context_switch(cur: &mut Context, nxt: &Context);
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(i32)]
pub enum ThreadState {
    /// 可运行
    Runnable,
    /// 阻塞，等待被唤醒
    Blocking,
    /// 异步等待
    Waiting,
    /// 已退出，但尚不能回收
    Zombie,
    /// 已退出, 可以被回收
    Waited,
}

#[cfg(debug_assertions)]
pub const THREAD_SIZE: usize = 32768;

#[cfg(not(debug_assertions))]
pub const THREAD_SIZE: usize = 8192;

#[cfg(debug_assertions)]
#[repr(align(32768))]
struct ThreadAlign;

#[repr(C)]
pub struct Thread {
    _align: ThreadAlign,
    pub tid: usize,
    pub proc: ProcPtr,
    pub state: ThreadState,
    pub exit_code: i32,
    pub ctx: Context,
    kstack: [u8; THREAD_SIZE - size_of::<usize>() * 3 - size_of::<Context>()],
}

pub type ThreadPtr = &'static mut Thread;

/// 用户线程的入口
///
/// syscall_return定义在中断模块trap.S中
pub fn user_task_entry(_: usize) -> usize {
    unsafe {
        syscall_return(current().syscall_frame());
    }
}

impl Thread {
    /// 创建一个新的线程
    ///
    /// 返回（线程指针，是否需要栈）
    pub fn new(proc: &mut Process, entry: fn(usize) -> usize, arg: usize) -> (ThreadPtr, bool) {
        // 线程入口函数
        fn kernel_thread_entry() -> ! {
            let cur = current();
            let entry: fn(usize) -> usize = unsafe { transmute(cur.ctx.regs.rbx) };
            let arg = cur.ctx.regs.rbp;
            let ret = entry(arg);
            // 若是用户态线程，则不会执行下面的exit，需要手动在线程函数中exit
            cur.exit(ret as _);
        }
        let (t, need_stack);
        unsafe {
            let mut it = proc.threads.iter_mut();
            loop {
                // 寻找是否有已退出可回收的线程
                if let Some(t1) = it.next() {
                    if t1.state == ThreadState::Waited {
                        t = transmute(t1);
                        need_stack = false;
                        break;
                    }
                // 遍历结束,没有Waited状态的线程
                } else {
                    let mut t1 = Box::<Thread>::new_uninit();
                    t = &mut *t1.as_mut_ptr();
                    t.tid = proc.threads.len();
                    proc.threads.push(transmute(t1));
                    need_stack = true;
                    break;
                }
            }
            THREAD_MANAGER.get().enqueue(&mut *(t as *mut _));
            t.proc = &mut *(proc as *mut _);
        }
        t.state = ThreadState::Runnable;
        t.ctx.rip = kernel_thread_entry as _;
        t.ctx.regs.rsp =
            t.kstack.as_ptr_range().end as usize - size_of::<usize>() - size_of::<SyscallFrame>();
        t.ctx.regs.rbx = entry as _;
        t.ctx.regs.rbp = arg;
        (t, need_stack)
    }

    /// 线程退出
    ///
    /// 在每个内核线程的入口函数中，执行完线程函数后调用
    pub fn exit(&mut self, exit_code: i32) -> ! {
        println!(
            "[kernel] Process {} Thread {} exited with code {}",
            self.proc.pid, self.tid, exit_code
        );
        // 为根线程，这时释放进程所有线程资源
        if self.tid == 0 {
            let p = &mut self.proc;
            PID2PROC.get().remove(&p.pid).unwrap();
            p.vm = None;
            p.zombie = true;
            p.exit_code = exit_code;
            for ch in &mut p.children {
                root_proc().add_child(ch);
            }
            p.children.clear();
            for t in &mut p.threads {
                t.state = ThreadState::Zombie;
            }
            THREAD_MANAGER.get().clear_zombie();
            // 清理除了0号线程的所有线程. 内核代码在0号进程中,故不能清理
            p.threads.drain(1..);
            p.files.clear();
        }
        self.exit_code = exit_code;
        self.state = ThreadState::Zombie;
        THREAD_MANAGER.get().resched();
        unreachable!("task exited");
    }

    /// 从当前线程栈最高地址获取SyscallFrame
    pub fn syscall_frame(&mut self) -> &mut SyscallFrame {
        unsafe { &mut *(self.kstack.as_ptr_range().end as *mut SyscallFrame).sub(1) }
    }

    /// 切换到下一个就绪线程
    pub fn switch_to(&mut self, nxt: &Thread) {
        if let Some(vm) = &nxt.proc.vm {
            vm.activate();
        }
        unsafe {
            context_switch(&mut self.ctx, &nxt.ctx);
        }
    }
}
