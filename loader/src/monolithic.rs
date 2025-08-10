//! 宏内核加载器

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::pin::Pin;
use hal::arch::cpu::MachineContext;
use hal::println;
use log::info;
use monolithic_objects::ROOT_INODE;
use monolithic_objects::Siginfo;
use monolithic_objects::SignalActionFlags;
use monolithic_objects::SignalFrame;
use monolithic_objects::SignalStackFlags;
use monolithic_objects::SignalUserContext;
use monolithic_objects::Sigset;
use monolithic_objects::set_current_thread;
use monolithic_objects::sync::timer;
use monolithic_objects::{Arc, Thread, ThreadState};
use num_traits::FromPrimitive;
use trapframe::TrapFrame;
use trapframe::UserContext;

const PAGE_FAULT: usize = 14;
const TIMER: usize = 32;

/// 加载运行第一个用户程序Shell
pub fn run_shell() {
    let shell = "sqlite-test";
    // let shell = "shell";
    info!("Trying to enter user shell now!");
    if let Ok(inode) = ROOT_INODE.lookup(shell) {
        let thread = Thread::new_user(
            &inode,
            shell,
            vec!["busybox".into(), "ash".into()],
            Vec::new(),
        )
        .expect("Failed to create shell.");
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
        if thread.inner.lock().state == ThreadState::Exited {
            break;
        }
        // 进入用户态
        let mut context = thread.begin_running();
        if let Some((idx, info, sigmask)) = thread.handle_signal() {
            let mut proc = thread.proc.lock();
            proc.signals.remove(idx);
            context = handle_signal(thread.clone(), context, info, sigmask);
        }

        context.run();
        // 返回内核，处理中断/系统调用
        handle_user_trap(thread.clone(), &mut context).await;
        thread.end_running(context);
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
            timer();
        }
        _ => {
            unimplemented!();
        }
    }
}

/// 处理用户态中断或系统调用
async fn handle_user_trap(thread: Arc<Thread>, ctx: &mut Box<UserContext>) {
    // 用户态系统调用
    if ctx.trap_num == 0x100 {
        let syscall_num = ctx.get_syscall_num();
        let args = ctx.get_syscall_args();
        let mut syscall = monolithic_syscalls::Syscall {
            thread: &thread,
            thread_fn: thread_fn,
            context: ctx,
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
            timer();
        }
        _ => {
            unimplemented!();
        }
    }
}

/// Handle signal of current thread.
/// May change the user context to run signal handler first.
fn handle_signal(
    thread: Arc<Thread>,
    mut ctx: Box<UserContext>,
    siginfo: Siginfo,
    _sigmask: Sigset,
) -> Box<UserContext> {
    pub const SIG_ERR: usize = usize::max_value() - 1;
    pub const SIG_DFL: usize = 0;
    pub const SIG_IGN: usize = 1;
    use monolithic_objects::Signal::*;

    let mut proc = thread.proc.lock();
    let signal = FromPrimitive::from_i32(siginfo.signo).unwrap();
    let action = proc.signal_action(signal);
    let action_flags = SignalActionFlags::from_bits_truncate(action.flags.bits());
    info!("thread {} received signal: {:?}", thread.tid, signal);
    // Enter signal handler.
    match action.handler {
        SIG_DFL => match signal {
            SIGALRM | SIGHUP | SIGINT => {
                info!("Default action: Term!");
                proc.exit(siginfo.signo as _);
                ctx
            }
            _ => ctx,
        },
        SIG_IGN => {
            info!("Ignore!");
            ctx
        }
        SIG_ERR => {
            unimplemented!()
        }
        _ => {
            info!("Go to handler at {:#x}", action.handler);
            // mask current signal and actions mask.
            let mut inner = thread.inner.lock();
            // store orignal sig_mask.
            let sig_mask = inner.signal_mask;
            // store orignal altstack.
            let stack = inner.signal_altstack;
            inner.signal_mask.add(signal);
            inner.signal_mask.add_set(&action.mask);
            drop(inner);

            let sig_sp: usize = {
                if action_flags.contains(SignalActionFlags::ONSTACK) {
                    let stack_flags = SignalStackFlags::from_bits_truncate(stack.flags);
                    if stack_flags.contains(SignalStackFlags::DISABLE) {
                        ctx.get_sp()
                    } else {
                        let mut inner = thread.inner.lock();
                        inner.signal_altstack.flags |= SignalStackFlags::ONSTACK.bits();

                        // handle auto disarm.
                        if stack_flags.contains(SignalStackFlags::AUTODISARM) {
                            inner.signal_altstack.flags |= SignalStackFlags::DISABLE.bits();
                        }
                        stack.sp + stack.size
                    }
                } else {
                    ctx.get_sp()
                }
            } - core::mem::size_of::<SignalFrame>();
            let frame: &'static mut SignalFrame = unsafe {
                let slice: &'static mut [SignalFrame] =
                    core::slice::from_raw_parts_mut(sig_sp as *mut SignalFrame, 1);
                &mut slice[0]
            };
            frame.info = siginfo;
            frame.ucontext = SignalUserContext {
                flags: 0,
                link: 0,
                stack,
                context: MachineContext::from_tf(&mut ctx),
                sig_mask,
            };
            if action_flags.contains(SignalActionFlags::RESTORER) {
                frame.ret_code_addr = action.restorer; // legacy
            } else {
                frame.ret_code_addr = frame.ret_code.as_ptr() as usize;
                // mov SYS_RT_SIGRETURN, %eax
                frame.ret_code.copy_from_slice(&RET_CODE);
            }
            let ctx = set_signal_handler(
                ctx,
                sig_sp,
                action.handler,
                siginfo.signo as _,
                &frame.info as *const Siginfo,
                &frame.ucontext as *const SignalUserContext,
            );
            ctx
        }
    }
}

/// Set signal handler.
pub fn set_signal_handler(
    mut ctx: Box<UserContext>,
    sp: usize,
    handler: usize,
    signo: usize,
    siginfo: *const Siginfo,
    uctx: *const SignalUserContext,
) -> Box<UserContext> {
    ctx.general.rsp = sp;
    ctx.general.rip = handler;

    // Pass handler argument.
    ctx.general.rdi = signo as usize;
    ctx.general.rsi = siginfo as usize;
    ctx.general.rdx = uctx as usize;
    ctx
}

/// mov 0x15 %eax(SYS_RT_SIGRETURN)
/// syscall
pub const RET_CODE: [u8; 7] = [
    // mov SYS_RT_SIGRETURN, %eax
    0xb8, 15, 0, 0, 0, //
    0x0f, 0x05, // syscall
];
