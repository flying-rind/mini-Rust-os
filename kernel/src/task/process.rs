use super::*;
use crate::{fs::*, mm::*, *};
use alloc::{collections::BTreeMap, rc::Rc};

pub type ProcPtr = &'static mut Process;

#[derive(Default)]
pub struct Process {
    pub pid: usize,
    pub zombie: bool,
    pub exit_code: i32,
    pub vm: Option<MemorySet>,
    pub parent: Option<ProcPtr>,
    pub children: Vec<ProcPtr>,
    pub threads: Vec<Box<Thread>>,
    pub files: Vec<Option<Rc<dyn File>>>,
}

pub(crate) fn new_id() -> usize {
    static NEXT_ID: Cell<usize> = zero();
    let next = *NEXT_ID + 1;
    *NEXT_ID.get() = next;
    next
}

/// PID到PROC的映射
pub static PID2PROC: Cell<BTreeMap<usize, ProcPtr>> = Cell::new(BTreeMap::new());

impl Process {
    /// 复制当前进程
    ///
    /// 目前只支持单线程进程
    pub fn fork(&mut self) -> ProcPtr {
        assert_eq!(self.threads.len(), 1);
        let child = Box::leak(Box::new(Process {
            pid: new_id(),
            vm: self.vm.clone(),
            files: self.files.clone(),
            ..Process::default()
        }));
        let t = unsafe {
            let child = child as *mut Process;
            PID2PROC.get().insert((*child).pid, &mut *child);
            self.add_child(&mut *child);
            Thread::new(&mut *child, user_task_entry, 0).0
        };
        let f = t.syscall_frame();
        // 复制根线程的syscall_frame
        *f = *self.threads[0].syscall_frame();
        // 设置子进程的返回值为0
        f.caller.rax = 0;
        child
    }

    /// 将当前进程的进程入口设置为elf文件入口
    ///
    /// 目前只支持单线程的进程
    pub fn exec(&mut self, path: &str, args: Vec<String>) -> isize {
        assert_eq!(self.threads.len(), 1);
        if let Some(file) = open_file(path, OpenFlags::RDONLY) {
            let elf_data = file.read_all();
            let (entry, vm) = mm::load_app(&elf_data);
            vm.activate();
            let mut top = (USTACK_TOP - (args.len() + 1) * size_of::<usize>()) as *mut u8;
            let argv = top as *mut usize;
            unsafe {
                for (i, arg) in args.iter().enumerate() {
                    top = top.sub(arg.len() + 1);
                    core::ptr::copy_nonoverlapping(arg.as_ptr(), top, arg.len());
                    // '\0'
                    *top.add(arg.len()) = 0;
                    *argv.add(i) = top as _;
                }
                // Set argv[argc] = NULL
                *argv.add(args.len()) = 0;
            }
            self.vm = Some(vm);
            let f = self.threads[0].syscall_frame();
            // 进入用户态的sysretq指令会从rcx中恢复ip
            // 从r11中恢复rflags
            f.caller.rcx = entry;
            f.caller.r11 = my_x86_64::RFLAGS_IF;
            f.callee.rsp = top as usize & !0xF;
            f.caller.rdi = args.len();
            f.caller.rsi = argv as _;
            0
        } else {
            -1
        }
    }

    ///  回收一个子进程，返回（子进程号，子进程退出码）
    ///
    /// 未找到子进程-> -1; 找到未回收-> -2;
    pub fn waitpid(&mut self, pid: isize) -> (isize, i32) {
        let mut found_pid = false;
        for (idx, p) in self.children.iter().enumerate() {
            // 若pid为-1，表示回收所有子线程
            if pid == -1 || p.pid == pid as usize {
                found_pid = true;
                if p.zombie {
                    let child = self.children.remove(idx);
                    let ret = (child.pid as _, child.exit_code);
                    unsafe {
                        // drop it
                        drop(Box::from_raw(child));
                    }
                    return ret;
                }
            }
        }
        (if found_pid { -2 } else { -1 }, 0)
    }

    /// 获取进程地址空间页表的起始地址
    pub fn root_pa(&self) -> PhysAddr {
        self.vm.as_ref().unwrap().pt.root_pa
    }

    /// 为当前进程添加一个子进程
    pub fn add_child(&mut self, child: &mut Process) {
        unsafe {
            child.parent = transmute(self as *mut _);
            self.children.push(transmute(child));
        }
    }

    /// 为当前进程添加一个文件
    pub fn add_file(&mut self, file: Rc<dyn File>) -> usize {
        for (i, f) in self.files.iter_mut().enumerate() {
            if f.is_none() {
                *f = Some(file);
                return i;
            }
        }
        self.files.push(Some(file));
        self.files.len() - 1
    }
}
