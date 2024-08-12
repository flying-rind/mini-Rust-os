//! 中断和用户态系统调用的处理入口

use crate::*;

use alloc::sync::Arc;
use trapframe::{TrapFrame, UserContext};
const PAGE_FAULT: usize = 14;
const TIMER: usize = 32;

#[no_mangle]
/// 中断处理入口，由汇编直接调用无需手动调用
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
        // 系统调用
        if context.trap_num == 0x100 {
            let thread = thread.unwrap();
            thread.do_syscall();
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
            if let Some(thread) = thread {
                // 时间片轮转
                thread.set_state(ThreadState::Suspended);
            // 内核时钟
            } else if let Some(_tf) = tf {
                // 当前内核线程主动调度
                Scheduler::yield_current_kthread();
            } else {
                panic!("Should never happen!");
            }
        }
        _ => {
            println!("[Trap Handler]: unknown trap!");
            panic!("unknown trap!");
        }
    }
}
