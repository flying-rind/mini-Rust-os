//! 宏内核加载器

use alloc::boxed::Box;
use monolithic_objects::{Arc, Thread, ThreadState};
use monolithic_objects::CURRENT_THREAD;
use trapframe::UserContext;

/// 用户线程入口
/// 
/// loop:
/// - 进入用户态
/// - 处理中断/系统调用
async fn run_user(thread: Arc<Thread>) {
    {
        let cur_thread = CURRENT_THREAD.lock();
        *cur_thread = Some(thread.clone());
    }
    loop {
        if thread.state == ThreadState::Exited {
            break;
        }
        // TODO: Handle Signal
        // 进入用户态
        let mut ctx = thread.inner.lock().context.take().unwrap();
        ctx.run();
        // 处理中断/系统调用
        handle_user_trap(thread.clone(), ctx);
    }
    {
        let cur_thread = CURRENT_THREAD.lock();
        *cur_thread = None;
    }
}

/// 处理用户态中断或系统调用
async fn handle_user_trap(thread: Arc<Thread>, mut ctx: Box<UserContext>) {
    // 用户态系统调用
    if ctx.trap_num == 0x100 {
        let syscall_num = ctx.get_syscall_num();
        let args = ctx.get_syscall_args();
        // TODO:好像需要把上下文放回context
        let mut syscall = monolithic_syscalls::Syscall {
            thread: &thread,
            context: &mut *ctx,
        };
        let ret = syscall.syscall(num, args).await;
        ctx.set_syscall_ret(ret, 0);
        return;
    }

    // 内核或用户中断
    match ctx.trap_num {
        _ => {
            unimplemented!();
        }
    }
}