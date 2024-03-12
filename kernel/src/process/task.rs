use super::TASK_MANAGER;
use crate::{app::loader, mem::memory_set::*, trap::*, *};
use alloc::boxed::Box;
use core::{
    arch::global_asm,
    mem::{size_of, transmute},
};

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct Context {
    pub regs: CalleeRegs,
    pub rip: usize,
}

global_asm!(include_str!("switch.S"));

extern "C" {
    pub fn context_switch(cur: &mut Context, nxt: &Context);
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(i32)] // 为了对齐
pub enum TaskStatus {
    UnInit = 0,
    Runnable,
    Exited,
}

pub const TASK_SIZE: usize = 8192; // 32KB 一个任务结构体（PCB？）的最大大小

pub fn current() -> &'static mut Task {
    unsafe { &mut *((my_x86_64::read_rsp() & !(TASK_SIZE - 1)) as *mut Task) }
}

pub fn current_exit(exit_code: i32) -> ! {
    let cur = current();
    serial_println!("[kernel] Task {} exited with code {}", cur.id, exit_code);
    cur.status = TaskStatus::Exited;
    TASK_MANAGER.get().resched();
    unreachable!("task exited!");
}

pub fn current_yield() {
    TASK_MANAGER.get().resched();
}

#[repr(C, align(8192))] // TASK_SIZE
pub struct Task {
    pub id: usize,
    pub status: TaskStatus,
    pub ctx: Context,
    pub memory_set: Option<MemorySet>,
    pub kstack: [u8; TASK_SIZE
        - size_of::<usize>()
        - size_of::<TaskStatus>()
        - size_of::<Context>()
        - size_of::<Option<MemorySet>>()],
}

impl Task {
    pub fn new_kernel(id: usize, entry: fn(usize) -> usize, arg: usize) -> Box<Self> {
        fn kernel_task_entry() -> ! {
            let cur = current();
            let entry: fn(usize) -> usize = unsafe { transmute(cur.ctx.regs.rbx) };
            let arg = cur.ctx.regs.rbp;
            let ret = entry(arg);
            current_exit(ret as _);
        }
        let mut t = Box::<Task>::new_uninit();
        let p = unsafe { &mut *t.as_mut_ptr() };
        p.id = id;
        p.status = TaskStatus::Runnable;
        p.ctx.rip = kernel_task_entry as _;
        p.ctx.regs.rsp = p.kstack.as_ptr_range().end as usize - size_of::<usize>();
        p.ctx.regs.rbx = entry as _;
        p.ctx.regs.rbp = arg;
        unsafe {
            (&mut p.memory_set as *mut Option<MemorySet>).write(None);
            t.assume_init()
        }
    }

    pub fn new_user(id: usize, entry: usize, ms: MemorySet) -> Box<Self> {
        fn user_task_entry(entry: usize) -> usize {
            let cur = current();
            unsafe {
                let f = &mut *((cur.kstack.as_ptr_range().end as *mut SyscallFrame).sub(1));
                f.regs.rcx = entry;
                f.regs.r11 = my_x86_64::RFLAGS_IF;
                f.rsp = loader::USTACK_TOP;
                syscall_return(f);
            }
        }
        let mut t = Self::new_kernel(id, user_task_entry, entry);
        t.memory_set = Some(ms);
        t
    }

    pub fn switch_to(&mut self, nxt: &Task) {
        if let Some(ms) = &nxt.memory_set {
            ms.activate(); // user task
        }
        unsafe {
            context_switch(&mut self.ctx, &nxt.ctx);
        }
    }
}
