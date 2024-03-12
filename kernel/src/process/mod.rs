pub mod manager;
pub mod task;

use crate::{app::loader, my_x86_64, Cell};
use alloc::boxed::Box;
use core::mem::{size_of, transmute};
use manager::TaskManager;
use task::*;
// use alloc::vec::Vec;

static TASK_MANAGER: Cell<TaskManager> = unsafe { transmute([1u8; size_of::<TaskManager>()]) };

pub fn init() -> ! {
    // assert_eq!(size_of::<Task>(), TASK_SIZE);
    let m = TASK_MANAGER.get();
    let kernel_task_count = 3;
    m.tasks.push(Task::new_kernel(
        0,
        |_| {
            // running idle.
            loop {
                my_x86_64::enable_interrupts_and_hlt();
            }
        },
        0,
    ));
    m.tasks.push(Task::new_kernel(
        1,
        |arg| {
            serial_println!("test kernel task 0: arg = {:#x}", arg);
            0
        },
        0xdead,
    ));
    m.tasks.push(Task::new_kernel(
        2,
        |arg| {
            serial_println!("test kernel task 1: arg = {:#x}", arg);
            0
        },
        0xbeef,
    ));
    for i in 0..loader::get_app_count() {
        let (entry, ms) = loader::load_app(i);
        m.tasks
            .push(Task::new_user(i + kernel_task_count, entry, ms));
    }
    unsafe {
        context_switch(&mut Context::default(), &m.tasks[0].ctx);
    }
    unreachable!();
}
