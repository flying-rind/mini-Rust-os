//! 混合内核中断系统调用处理
//! 中断和用户态系统调用的处理入口

use alloc::string::ToString;
use alloc::sync::Arc;
use core::pin::Pin;
use hybrid_objects::fs::ROOT_INODE;
use hybrid_objects::task::Thread;
use hybrid_objects::*;
use log::error;
use log::info;
use trapframe::{TrapFrame, UserContext};

const PAGE_FAULT: usize = 14;
const TIMER: usize = 32;

/// 加载运行第一个用户程序Shell
pub fn run_shell() {
    let shell = "shell";
    println!("Running hybrid kernel now!");
    info!("Trying to enter user shell now!");
    if let Ok(inode) = ROOT_INODE.lookup(shell) {
        // Debug
        info!("Found shell");
        let args = vec!["arg1".to_string(), "arg2".to_string()];
        let envs = vec!["env1".to_string(), "env2".to_string()];
        let thread = Process::new_user(shell.to_string(), &inode, args, envs)
            .expect("Failed to create root thread!");
        thread.start(thread_fn);
    } else {
        panic!("Failed to load shell");
    }
}

/// 用户线程入口
///
/// loop:
/// - Get UserContext
/// - 进入用户态
/// - 处理中断/系统调用
/// - Put back UserContext
async fn run_user(thread: Arc<Thread>) {
    set_current_thread(Some(thread.clone()));
    loop {
        if thread.state() == ThreadState::Exited {
            break;
        }
        // Enter userspace until trap.
        thread.run_until_trap();
        // 返回内核，处理中断/系统调用
        handle_user_trap(thread.clone(), &mut thread.user_context()).await;
    }
    set_current_thread(None);
}

/// 用户线程统一线程函数
fn thread_fn(thread: Arc<Thread>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
    Box::pin(run_user(thread))
}

#[unsafe(no_mangle)]
/// 内核态中断处理入口，由汇编直接调用无需手动调用
pub extern "C" fn trap_handler(tf: &mut TrapFrame) {
    match tf.trap_num {
        PAGE_FAULT => {
            error!("[Trap Handler]: PAGEFAULT",);
            panic!("page fault");
        }
        TIMER => {
            // 当前内核线程主动调度
            Scheduler::yield_current_kthread();
        }
        _ => {
            unimplemented!();
        }
    }
}

/// 处理用户态的中断或系统调用
/// 若是系统调用则context中的trap_num一定为100
/// 若是中断则trap_num从context中获取
pub async fn handle_user_trap(thread: Arc<Thread>, context: &mut UserContext) {
    // 用户态系统调用
    if context.trap_num == 0x100 {
        let sys_num = context.get_syscall_num();
        let sys_args = context.get_syscall_args();
        let mut syscall = hybrid_syscalls::Syscall {
            thread: &thread,
            thread_fn,
            context,
        };
        let ret = syscall.do_syscall(sys_num, sys_args).await;
        thread.set_syscall_ret(ret as _, 0);
        return;
    }
    // 用户态中断
    match context.trap_num {
        // 页错误，目前直接panic
        PAGE_FAULT => {
            error!(
                "[Trap Handler]: PAGEFAULT, memory_set root_pa: {:x}",
                current_proc().memory_set().page_table().get().paddr()
            );
            panic!("page fault");
        }
        // 用户时钟中断
        TIMER => {
            pic::ack();
            *pic::TICKS.get_mut() += 1;
        }
        _ => {
            error!("[Trap Handler]: Unknown trap!");
            panic!("Unknown trap!");
        }
    }
}

/// 调度用户线程和内核线程
pub fn main_loop() {
    info!("[Kernel] Starting main loop...");
    loop {
        // 优先运行内核线程
        if let Some(kthread) = Scheduler::get_first_kthread() {
            let current_kthread = current_kthread();
            current_kthread.switch_to(current_kthread.clone(), kthread);
        } else {
            executor::run_util_idle();
            Scheduler::yield_current_kthread();
        }
    }
}
