mod manager;
mod task;

use crate::*;
use manager::TaskManager;
use task::*;

static TASK_MANAGER: Cell<TaskManager> = unsafe { transmute([1u8; size_of::<TaskManager>()]) };
static ROOT_TASK: Cell<usize> = zero();

pub fn init() -> ! {
    assert_eq!(size_of::<Task>(), TASK_SIZE);
    let mut m = TaskManager::new();
    m.enqueue(new_kernel(
        |_| {
            let cur = current();
            loop {
                my_x86_64::disable_interrupts();
                cur.waitpid(-1);
                my_x86_64::enable_interrupts_and_hlt();
            }
        },
        0,
    ));

    m.enqueue(new_kernel(
        |arg| {
            serial_println!("test kernel task 0: arg = {:#x}", arg);
            0
        },
        0xdead,
    ));

    m.enqueue(new_kernel(
        |arg| {
            serial_println!("test kernel task 1: arg = {:#x}", arg);
            0
        },
        0xbeef,
    ));

    let (entry, vm) = loader::load_app(loader::get_app_data_by_name("user_shell").unwrap());
    m.enqueue(new_user(entry, vm));
    let root = m.dequeue();
    unsafe {
        *ROOT_TASK.get() = root as *mut _ as _;
        (TASK_MANAGER.get() as *mut TaskManager).write(m);
        context_switch(&mut Context::default(), &root.ctx);
    }
    unreachable!();
}

pub fn root_task() -> &'static mut Task {
    unsafe { transmute(*ROOT_TASK) }
}

pub fn current() -> &'static mut Task {
    unsafe { &mut *((my_x86_64::read_rsp() & !(TASK_SIZE - 1)) as *mut Task) }
}

pub fn current_yield() {
    TASK_MANAGER.get().resched();
}
