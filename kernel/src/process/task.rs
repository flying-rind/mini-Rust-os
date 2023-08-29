use core::{
    arch::global_asm,
    mem::{size_of, transmute},
};

use alloc::boxed::Box;

use crate::process::{current, proc::PID2PROC, root_proc, TASK_MANAGER};

use super::proc::{Proc, ProcPtr};

use crate::trap::*;
// use core::mem::size_of;

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
    Runnable,
    Blocking,
    // Exited, but not waited, need to keep it.
    Zombie,
    // Exited and waited, can be recycled.
    Waited,
}

pub const TASK_SIZE: usize = 32768; // 32KB 一个任务结构体（PCB？）的最大大小

#[repr(align(32768))]
struct TaskAlign; // 不占大小，为了后面task结构体对齐

#[repr(C)]
pub struct Task {
    _align: TaskAlign,
    pub tid: usize,
    pub proc: ProcPtr,
    pub status: TaskStatus,
    pub exit_code: i32,
    pub ctx: Context,
    // kstack: [u8; TASK_SIZE - sizeof::<usize>() * 3 - size_of::<Context>()],
    kstack: [u8; TASK_SIZE - size_of::<usize>() * 3 - size_of::<Context>()],
}

pub type TaskPtr = &'static mut Task;

pub fn user_task_entry(_: usize) -> usize {
    unsafe {
        syscall_return(current().syscall_frame());
    }
}

impl Task {
    pub fn new(proc: &mut Proc, entry: fn(usize) -> usize, arg: usize) -> (TaskPtr, bool) {
        fn kernel_task_entry() -> ! {
            let cur = current();
            let entry: fn(usize) -> usize = unsafe { transmute(cur.ctx.regs.rbx) };
            let arg = cur.ctx.regs.rbp;
            let ret = entry(arg);
            cur.exit(ret as _);
        }

        let (t, need_stack);
        unsafe {
            let mut it = proc.tasks.iter_mut();
            loop {
                if let Some(t1) = it.next() {
                    if t1.status == TaskStatus::Waited {
                        t = transmute(t1);
                        need_stack = false;
                        break;
                    }
                } else {
                    let mut t1 = Box::<Task>::new_uninit();
                    t = &mut *t1.as_mut_ptr();
                    t.tid = proc.tasks.len();
                    proc.tasks.push(transmute(t1));
                    need_stack = true;
                    break;
                }
            }
            TASK_MANAGER.get().enquene(&mut *(t as *mut _));
            t.proc = &mut *(proc as *mut _);
        }

        t.status = TaskStatus::Runnable;
        t.ctx.rip = kernel_task_entry as _;
        t.ctx.regs.rsp =
            t.kstack.as_ptr_range().end as usize - size_of::<usize>() - size_of::<SyscallFrame>();
        t.ctx.regs.rbx = entry as _;
        t.ctx.regs.rbp = arg;
        (t, need_stack)
    }

    pub fn exit(&mut self, exit_code: i32) -> ! {
        serial_println!(
            "[kernel] Proc {} task {} exited with code {}",
            self.proc.pid,
            self.tid,
            exit_code
        );
        if self.tid == 0 {
            let p = &mut self.proc;
            PID2PROC.get().remove(&p.pid).unwrap();
            p.vm = None;
            p.zombie = true;
            p.exit_code = exit_code;
            for ch in &mut p.children {
                root_proc().add_child(ch);
            }
            p.children.clear();
            for t in &mut p.tasks {
                t.status = TaskStatus::Zombie;
            }
            TASK_MANAGER.get().clear_zombie();
            // clear zombie timer
            p.tasks.drain(1..);
        }
        self.exit_code = exit_code;
        self.status = TaskStatus::Zombie;
        TASK_MANAGER.get().resched();
        unreachable!("Task exited!");
    }

    pub fn switch_to(&mut self, nxt: &Task) {
        // if let Some
        unsafe {
            context_switch(&mut self.ctx, &nxt.ctx);
        }
    }

    pub fn syscall_frame(&mut self) -> &mut SyscallFrame {
        unsafe { &mut *(self.kstack.as_ptr_range().end as *mut SyscallFrame).sub(1) }
    }
}
