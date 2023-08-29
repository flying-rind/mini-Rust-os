pub mod manager;
pub mod proc;
pub mod task;

use core::ops::DerefMut;
// use mem;
use core::mem::{self, size_of, transmute};
use core::{cell::UnsafeCell, ops::Deref};

use task::{TaskPtr, TASK_SIZE};

use crate::process::proc::{new_id, Proc};
use crate::process::task::Context;
use crate::process::task::{context_switch, Task};

use self::manager::TaskManager;
use self::proc::ProcPtr;

use alloc::boxed::Box;
use alloc::vec::Vec;

#[inline(always)]
pub const fn zero<T>() -> T {
    unsafe { mem::MaybeUninit::zeroed().assume_init() }
}

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Cell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for Cell<T> {}

impl<T> Cell<T> {
    /// User is responsible to guarantee that inner struct is only used in
    /// uniprocessor.
    #[inline(always)]
    pub const fn new(val: T) -> Self {
        Self(UnsafeCell::new(val))
    }

    #[inline(always)]
    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

impl<T> Deref for Cell<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for Cell<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get()
    }
}

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
            // loop {
            unsafe {
                core::arch::asm!("cli");
                // waitpid
                // serial_println!("[proc 0] kfc crazy Thirsday v me 50");
                serial_println!("[proc {}] kfc crazy Thirsday v me 50", cur.proc.pid);
                fb_println!("[proc {}] kfc crazy Thirsday v me 50", cur.proc.pid);
                // cur.proc.waitpid(-1);
                core::arch::asm!("sti; hlt");
                TASK_MANAGER.get().resched();
                // serial_println!("task mc stone stopped.");
            }
            0x114514
            // }
        },
        0,
    );

    // let shell = root.fork();
    // shell.exec(5, Vec::new());
    let another_shell = root.fork();
    another_shell.exec("user_shell", Vec::new());
    unsafe {
        context_switch(&mut Context::default(), &TASK_MANAGER.get().dequene().ctx);
    }
    unreachable!();
}

pub fn current_yield() {
    TASK_MANAGER.get().resched();
}
