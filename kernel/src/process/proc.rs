use super::task::Task;
use super::zero;
use super::Cell;
use crate::app::loader::get_app_data_by_name;
use crate::app::loader::load_app;
use crate::app::loader::USTACK_TOP;
use crate::mem::memory_set::MemorySet;
use crate::process::task::user_task_entry;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::size_of;
use core::mem::transmute;

#[derive(Default)]
pub struct Proc {
    pub pid: usize,
    // pub signal: SignalFlags,
    pub zombie: bool,
    pub exit_code: i32,
    pub vm: Option<MemorySet>,
    // pub vm: Option<MemoryRegion>,
    pub parent: Option<ProcPtr>,
    pub children: Vec<ProcPtr>,
    pub tasks: Vec<Box<Task>>,
    // pub files: Vec<Option<Rc<dyn File>>>,
    // pub mutexes: Vec<Box<dyn Mutex>>,
    // pub sems: Vec<Sem>,
    // pub condvars: Vec<Condvar>,
}

pub type ProcPtr = &'static mut Proc;

pub fn new_id() -> usize {
    static NEXT_ID: Cell<usize> = zero();
    let next = *NEXT_ID + 1;
    *NEXT_ID.get() = next;
    next
}

pub static PID2PROC: Cell<BTreeMap<usize, ProcPtr>> = Cell::new(BTreeMap::new());

impl Proc {
    pub fn fork(&mut self) -> ProcPtr {
        assert_eq!(self.tasks.len(), 1);
        let child = Box::leak(Box::new(Proc {
            pid: new_id(),
            vm: self.vm.clone(),
            ..Proc::default()
        }));

        let t = unsafe {
            let child = child as *mut Proc;
            PID2PROC.get().insert((*child).pid, &mut *child);
            self.add_child(&mut *child);
            Task::new(&mut *child, user_task_entry, 0).0
        };

        let f = t.syscall_frame();
        *f = *self.tasks[0].syscall_frame();
        f.caller.rax = 0;

        child
    }

    pub fn exec(&mut self, path: &str, args: Vec<String>) -> isize {
        let (entry, vm) = load_app(get_app_data_by_name(path).unwrap());
        vm.activate();
        let mut top = (USTACK_TOP - (args.len() + 1) * size_of::<usize>()) as *mut u8;
        let argv = top as *mut usize;
        unsafe {
            for (_i, arg) in args.iter().enumerate() {
                top = top.sub(arg.len() + 1);
                core::ptr::copy_nonoverlapping(arg.as_ptr(), top, arg.len());
                *top.add(arg.len()) = 0;
                *argv.add(1) = top as _;
            }

            *argv.add(args.len()) = 0;
        }

        self.vm = Some(vm);
        let f = self.tasks[0].syscall_frame();
        f.caller.rcx = entry;
        f.caller.r11 = 0x200; // rflags
        f.callee.rsp = top as usize & !0xF; // for alignment
        f.caller.rdi = args.len();
        f.caller.rsi = argv as _;
        0
    }

    pub fn add_child(&mut self, child: &mut Proc) {
        unsafe {
            child.parent = transmute(self as *mut _);
            self.children.push(transmute(child));
        }
    }

    pub fn waitpid(&mut self, pid: isize) -> (isize, i32) {
        let mut found_pid = false;
        for (idx, p) in self.children.iter().enumerate() {
            if pid == -1 || p.pid == pid as usize {
                found_pid = true;
                if p.zombie {
                    let child = self.children.remove(idx);
                    let ret = (child.pid as _, child.exit_code);
                    unsafe {
                        Box::from_raw(child);
                    } // Drop it.
                    return ret;
                }
            }
        }
        (if found_pid { -2 } else { -1 }, 0)
    }
}
