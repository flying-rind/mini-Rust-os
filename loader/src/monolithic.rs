//! 宏内核加载器

use core::pin::Pin;
use core::str::FromStr;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use hal::println;
use log::info;
use monolithic_objects::ROOT_INODE;
use monolithic_objects::set_current_thread;
use monolithic_objects::{Arc, Thread, ThreadState};
use trapframe::TrapFrame;
use trapframe::UserContext;

const PAGE_FAULT: usize = 14;
const TIMER: usize = 32;

/// 加载运行第一个用户程序Shell
pub fn run_shell() {
    let shell = "app1";
    info!("Trying to enter user shell now!");
    if let Ok(inode) = ROOT_INODE.lookup(shell) {
        let thread = Thread::new_user(
            &inode,
            shell,
            vec![String::from_str("app1").unwrap()],
            Vec::new(),
        );
        let future = thread_fn(thread.clone());
        executor::spawn(future);
    } else {
        panic!("Failed to load shell");
    }
}

/// 用户线程入口
///
/// loop:
/// - 进入用户态
/// - 处理中断/系统调用
async fn run_user(thread: Arc<Thread>) {
    set_current_thread(Some(thread.clone()));
    loop {
        if thread.inner.lock().state == ThreadState::Exited {
            break;
        }
        // TODO: Handle Signal
        // 切换地址空间
        thread.proc.lock().vm.activate();
        // 进入用户态
        let mut ctx = thread.inner.lock().context.take().unwrap();
        ctx.run();
        // 返回内核，处理中断/系统调用
        handle_user_trap(thread.clone(), ctx).await;
    }
    set_current_thread(None);
}

/// 用户线程统一线程函数
fn thread_fn(thread: Arc<Thread>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
    Box::pin(run_user(thread))
}

/// 内核态中断处理入口，由汇编直接调用无需手动调用
#[unsafe(no_mangle)]
pub extern "C" fn trap_handler(tf: &mut TrapFrame) {
    match tf.trap_num {
        PAGE_FAULT => {
            println!("[Trap Handler]: PAGEFAULT",);
            panic!("page fault");
        }
        TIMER => {
            // do nothing?
        }
        _ => {
            unimplemented!();
        }
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
            thread_fn: thread_fn,
        };
        let ret = syscall.syscall(syscall_num, args).await;
        ctx.set_syscall_ret(ret as _, 0);
        return;
    }

    // 内核或用户中断
    match ctx.trap_num {
        PAGE_FAULT => {
            println!("[Trap Handler]: PAGEFAULT",);
            panic!("page fault");
        }
        TIMER => {
            // do nothing?
        }
        _ => {
            unimplemented!();
        }
    }
}
