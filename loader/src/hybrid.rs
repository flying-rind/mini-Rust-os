//! 混合内核中断系统调用处理
//! 中断和用户态系统调用的处理入口

use alloc::sync::Arc;
use hybrid_objects::task::Thread;
use hybrid_objects::*;
use trapframe::{TrapFrame, UserContext};
use hal::console::serial_print;

const PAGE_FAULT: usize = 14;
const TIMER: usize = 32;

#[unsafe(no_mangle)]
/// 内核态中断处理入口，由汇编直接调用无需手动调用
pub extern "C" fn trap_handler(tf: &mut TrapFrame) {
    handle_trap(Some(tf), None, None);
}

/// 处理用户态的中断或系统调用
/// 若是系统调用则context中的trap_num一定为100
/// 若是中断则trap_num从context中获取
pub fn handle_user_trap(thread: Arc<Thread>, context: &UserContext) {
    handle_trap(None, Some(thread), Some(context));
}

/// 中断/系统调用处理函数
pub fn handle_trap(
    tf: Option<&mut TrapFrame>,
    thread: Option<Arc<Thread>>,
    context: Option<&UserContext>,
) {
    // 用户态的中断或系统调用
    if let Some(context) = context {
        // 用户态系统调用
        if context.trap_num == 0x100 {
            let thread = thread.unwrap();
            let sys_num = context.get_syscall_num();
            let sys_args = context.get_syscall_args();
            let mut syscall = hybrid_syscalls::Syscall { thread: &thread };
            let (ret0, ret1) = syscall.do_syscall(sys_num, sys_args);
            thread.set_syscall_ret(ret0, ret1);
            return;
        }
    }

    // 处理用户态或内核态中断
    let trap_num = if tf.is_some() {
        // 内核中断
        tf.as_ref().unwrap().trap_num
    } else {
        // 用户中断
        context.unwrap().trap_num
    };
    match trap_num {
        // 页错误，目前直接panic
        PAGE_FAULT => {
            println!(
                "[Trap Handler]: PAGEFAULT, memory_set root_pa: {:x}",
                current_proc().memory_set().page_table().get().paddr()
            );
            panic!("page fault");
        }
        // 时钟中断，轮转用户线程或内核线程
        TIMER => {
            pic::ack();
            *pic::TICKS.get_mut() += 1;
            // 用户时钟
            if let Some(_thread) = thread {
                // 时间片轮转
                // thread.set_state(ThreadState::Suspended);
            // 内核时钟
            } else if let Some(_tf) = tf {
                // 当前内核线程主动调度
                Scheduler::yield_current_kthread();
            } else {
                panic!("Should never happen!");
            }
        }
        _ => {
            println!("[Trap Handler]: Unknown trap!");
            panic!("Unknown trap!");
        }
    }
}

/// 调度用户线程和内核线程
pub fn main_loop() {
    println!("[Kernel] Starting main loop...");
    loop {
        // 优先运行内核线程
        let kthread = Scheduler::get_first_kthread();
        if kthread.is_some() {
            // [Debug]
            // println!("`Root` switch to `{}`", kthread.as_ref().unwrap().name());
            // 将CPU交给服务线程或执行器
            let kthread = kthread.unwrap();
            let current_kthread = CURRENT_KTHREAD.get().as_ref().unwrap().clone();
            // 修改当前内核线程
            *CURRENT_KTHREAD.get_mut() = Some(kthread.clone());
            // 主线程入队
            KTHREAD_DEQUE.get_mut().push_back(current_kthread.clone());
            current_kthread.switch_to(kthread);
        } else {
            let uthread = Scheduler::get_first_uthread();
            // 运行用户线程
            if uthread.is_some() {
                let uthread = uthread.unwrap();
                // 修改当前线程
                *CURRENT_THREAD.get_mut() = Some(uthread.clone());
                // 持续运行用户线程直到其被挂起
                // [Debug]
                // println!("uthread running, pid {}", uthread.proc().unwrap().pid());
                while uthread.state() == ThreadState::Runnable {
                    uthread.run_until_trap();
                    handle_user_trap(uthread.clone(), &uthread.user_context());
                }
                // 此时线程已被挂起
                clear_current_thread();
            }
        }
    }
}

/// 清理当前线程
pub fn clear_current_thread() {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    // 根据线程状态进行清理
    match current_thread.state() {
        ThreadState::Suspended => {
            current_thread.set_state(ThreadState::Runnable);
            THREAD_DEQUE.get_mut().push_back(current_thread.clone());
        }
        ThreadState::Runnable | ThreadState::Waiting | ThreadState::Stop => {
            THREAD_DEQUE.get_mut().push_back(current_thread.clone());
        }
        ThreadState::Exited => {
            // 已退出时清理当前线程全局变量以drop线程
            current_thread.exit();
            *CURRENT_THREAD.get_mut() = None;
        }
    }
}


