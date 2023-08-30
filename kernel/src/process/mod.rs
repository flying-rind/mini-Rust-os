pub mod manager;
pub mod proc;
pub mod task;

use crate::my_x86_64;
use crate::process::proc::{new_id, Proc};
use crate::process::task::Context;
use crate::process::task::{context_switch, Task};
use crate::zero;
use crate::Cell;
use core::mem::{size_of, transmute};
use task::{TaskPtr, TASK_SIZE};

use self::manager::TaskManager;
use self::proc::ProcPtr;

use alloc::boxed::Box;
use alloc::vec::Vec;

static ROOT_PROC: Cell<usize> = zero();
static TASK_MANAGER: Cell<TaskManager> = unsafe { transmute([1u8; size_of::<TaskManager>()]) };

pub fn current() -> TaskPtr {
    // unsafe { &mut *((x86_64::read_rsp() & !(TASK_SIZE - 1)) as *mut _) }
    unsafe {
        let val: usize; // rsp
        core::arch::asm!(
            // "nop",
            "mov {}, rsp",
            out(reg) val
        );

        &mut *((val & !(TASK_SIZE - 1)) as *mut _)
    }
}

pub fn root_proc() -> ProcPtr {
    unsafe { transmute(*ROOT_PROC) }
}

pub fn init() -> ! {
    assert_eq!(size_of::<Task>(), TASK_SIZE);
    unsafe {
        (TASK_MANAGER.get() as *mut TaskManager).write(TaskManager::default());
    }

    let root = Box::leak(Box::new(Proc {
        pid: new_id(),
        ..Proc::default()
    }));
    *ROOT_PROC.get() = root as *mut _ as _;
    Task::new(
        root,
        |_| {
            let cur = current();
            // Running idle and recycle orphans.
            loop {
                my_x86_64::disable_interrupts();
                serial_println!("Root proc waitpid");
                cur.proc.waitpid(-1);
                my_x86_64::enable_interrupts_and_hlt();
            }
        },
        0,
    );

    let shell = root.fork();
    shell.exec("00hello_world", Vec::new());
    unsafe {
        context_switch(&mut Context::default(), &TASK_MANAGER.get().dequene().ctx);
    }
    unreachable!();
}

pub fn current_yield() {
    TASK_MANAGER.get().resched();
}
