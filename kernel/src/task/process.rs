//! 进程抽象
use crate::{mm::*, *};

use alloc::sync::Arc;
use alloc::sync::Weak;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::usize;
use fs::OpenFlags;
use fs::{open_file, File, Stdin, Stdout};
use hashbrown::HashMap;
use spin::Lazy;
use spin::RwLock;
use sync::{Condvar, MutexBlocking, Sem};
use x86_64::structures::paging::PageTableFlags;
use xmas_elf::ElfFile;

/// 全局变量，PID到进程对象的映射
pub static PROCESS_MAP: Lazy<Cell<HashMap<usize, Arc<Process>>>> =
    Lazy::new(|| Cell::new(HashMap::new()));

/// 全局变量：进程ID，用于创建进程时分配ID
pub static PROCESS_ID: AtomicUsize = AtomicUsize::new(0);

/// 进程抽象
#[derive(Default)]
pub struct Process {
    /// 进程id
    pid: usize,
    /// 进程名称，可重复
    name: String,
    /// 进程地址空间
    memory_set: Arc<MemorySet>,
    /// 父进程
    parent: RwLock<Weak<Process>>,
    /// 子进程队列
    children: Cell<Vec<Arc<Process>>>,
    /// 进程包含的线程集合，TID到线程对象的映射
    threads: Cell<HashMap<usize, Arc<Thread>>>,
    /// 线程ID，创建新线程时分配
    thread_id: AtomicUsize,
    /// 文件表
    file_table: Cell<Vec<Option<Arc<dyn File>>>>,
    /// 互斥锁
    mutexes: Cell<Vec<Arc<MutexBlocking>>>,
    /// 信号量
    sems: Cell<Vec<Arc<Sem>>>,
    /// 条件变量
    condvars: Cell<Vec<Arc<Condvar>>>,
}

impl Process {
    /// 创建新进程
    ///
    /// 为其创建虚存空间，将elf文件载入虚存空间中，并建立根线程
    ///
    /// 若路径有误，则返回None
    pub fn new(name: String, path: &str, args: Option<Vec<String>>) -> Option<Arc<Self>> {
        // 从文件系统中读取elf文件，载入到地址空间
        let file = match open_file(path, OpenFlags::RDONLY) {
            Some(file) => file,
            // 路径有误，直接返回None
            None => return None,
        };
        // 新建进程虚存空间
        let memory_set = MemorySet::new();
        let elf_data = file.read_all();
        let elf = ElfFile::new(&elf_data).unwrap();
        load_app(memory_set.clone(), &elf);
        let entry = elf.header.pt2.entry_point() as usize;

        // 为根线程创建用户栈内存区域
        let stack_area = MemoryArea::new(
            USER_STACK_BASE,
            USER_STACK_SIZE,
            PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE | PageTableFlags::PRESENT,
            MemAreaType::USERSTACK,
        );
        memory_set.insert_area(stack_area.clone());
        // 切换到新进程地址空间以访问其用户栈
        memory_set.activate();
        // 参数压栈
        let (stack_top, argc, argv) = if args.is_none() {
            (USER_STACK_BASE + USER_STACK_SIZE, 0, 0)
        } else {
            push_to_stack(stack_area.clone(), args)
        };
        // 构造新进程，默认打开标准输入输出
        let pid = PROCESS_ID.fetch_add(1, Ordering::Relaxed);
        let new_proc = Arc::new(Process {
            pid,
            name,
            memory_set,
            file_table: Cell::new(vec![
                Some(Arc::new(Stdin)),
                Some(Arc::new(Stdout)),
                Some(Arc::new(Stdout)),
            ]),
            ..Process::default()
        });

        // 加入全局进程映射表
        PROCESS_MAP.get_mut().insert(pid, new_proc.clone());

        // 创建根线程
        let tid = new_proc.thread_id.fetch_add(1, Ordering::Relaxed);
        let root_thread = Thread::new(
            Arc::downgrade(&new_proc),
            tid,
            entry,
            stack_top,
            argc,
            argv,
            stack_area,
        );
        new_proc.add_thread(root_thread);
        Some(new_proc)
    }

    /// 复制当前进程
    ///
    /// 若但前进程有多个线程则只复制当前线程
    pub fn fork(&self) -> Arc<Process> {
        assert_eq!(self.threads.len(), 1);
        let pid = PROCESS_ID.fetch_add(1, Ordering::Relaxed);
        // 创建子进程复制父进程的文件表和地址空间（不包括用户栈）
        let memory_set = self.memory_set.clone_myself();
        let child_proc = Arc::new(Process {
            pid,
            name: self.name.clone(),
            memory_set: memory_set.clone(),
            file_table: Cell::new(self.file_table.get().clone()),
            ..Process::default()
        });
        // 加入全局进程映射表
        PROCESS_MAP.get_mut().insert(pid, child_proc.clone());
        // 复制用户栈
        let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
        let current_proc = current_thread.proc().unwrap();
        let new_stack_area = current_thread.stack_area().clone_myself();
        memory_set.insert_area(new_stack_area.clone());
        let current_ctx = current_thread.user_context();
        // 创建根线程
        let tid = child_proc.alloc_tid();
        let root_thread = Thread::new(
            Arc::downgrade(&child_proc.clone()),
            tid,
            0,
            0,
            0,
            0,
            new_stack_area,
        );
        // 复制上下文
        root_thread.set_user_context(current_ctx);
        // 子线程返回值为0
        root_thread.set_rax(0);
        root_thread.set_state(ThreadState::Runnable);
        child_proc.add_thread(root_thread);
        child_proc.set_parent(Arc::downgrade(&current_proc));
        self.add_child(child_proc.clone());
        child_proc
    }

    /// 替换当前进程的elf文件
    pub fn exec(&self, path: &str, args: Option<Vec<String>>) -> usize {
        if let Some(file) = open_file(path, OpenFlags::RDONLY) {
            let elf_data = file.read_all();
            let elf = ElfFile::new(&elf_data).unwrap();
            // 清理除了exec之外的所有子线程
            let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
            for (_, thread) in self.threads.get() {
                if current_thread.tid() != thread.tid() {
                    thread.set_state(ThreadState::Exited);
                }
            }
            let threads = self.threads.get_mut();
            threads.clear();
            threads.insert(current_thread.tid(), current_thread.clone());
            // 清理地址空间之前的elf虚存区域
            self.memory_set.clear_elf();
            // 重新加载elf
            load_app(self.memory_set.clone(), &elf);
            let entry = elf.header.pt2.entry_point() as usize;
            // 参数压栈
            self.memory_set.activate();
            let (stack_top, argc, argv) = if args.is_none() {
                (USER_STACK_BASE + USER_STACK_SIZE, 0, 0)
            } else {
                push_to_stack(current_thread.stack_area(), args)
            };
            // 准备根线程现场
            // println!("entry: {:x}", entry);
            current_thread.set_ip(entry);
            current_thread.set_sp(stack_top);
            current_thread.set_args(argc, argv);
            return 1;
        }
        usize::MAX
    }

    /// 退出进程
    pub fn exit(&self) {
        // 退出所有线程
        for (_tid, thread) in self.threads.clone().into_iter() {
            // 线程应该是被调度器exit()的
            thread.set_state(ThreadState::Exited)
        }
        // 从全局进程映射和父进程中删除自己的引用
        PROCESS_MAP.get_mut().remove(&self.pid);
        if let Some(parent_proc) = self.parent() {
            parent_proc.remove_child(self.pid);
        }
        // 是否需要手动drop？
        // drop(self.memory_set);
        self.children.get_mut().drain(..);
        // 无需清理自己的线程队列，线程被调度器清理的时候会清除进程对自己的引用
        // self.threads.get_mut().drain();
    }

    /// 为当前进程添加一个子进程
    pub fn add_child(&self, process: Arc<Process>) {
        self.children.get_mut().push(process);
    }

    /// 分配一个线程id
    pub fn alloc_tid(&self) -> usize {
        self.thread_id.fetch_add(1, Ordering::Relaxed)
    }

    /// 增加一个线程
    pub fn add_thread(&self, thread: Arc<Thread>) {
        self.threads.get_mut().insert(thread.tid(), thread);
    }

    /// 增加一个文件，返回新增文件的fd
    ///
    /// 若有已经关闭的文件，则使用当前文件替换
    ///
    /// 以此实现dup和管道
    pub fn add_file(&self, new_file: Arc<dyn File>) -> usize {
        let file_table = self.file_table.get_mut();
        for (i, file) in file_table.iter_mut().enumerate() {
            if file.is_none() {
                *file = Some(new_file);
                return i;
            }
        }
        file_table.push(Some(new_file));
        file_table.len() - 1
    }

    /// 增加一个互斥锁，返回ID
    pub fn add_mutex(&self, mutex: Arc<MutexBlocking>) -> usize {
        let mutexes = self.mutexes.get_mut();
        mutexes.push(mutex);
        mutexes.len() - 1
    }

    /// 增加一个信号量，返回ID
    pub fn add_sem(&self, sem: Arc<Sem>) -> usize {
        let sems = self.sems.get_mut();
        sems.push(sem);
        sems.len() - 1
    }

    /// 增加一个条件变量，返回ID
    pub fn add_condvar(&self, condvar: Arc<Condvar>) -> usize {
        let condvars = self.condvars.get_mut();
        condvars.push(condvar);
        condvars.len() - 1
    }

    /// 设置父进程
    pub fn set_parent(&self, proc: Weak<Process>) {
        *self.parent.write() = proc;
    }

    /// 获取进程名称
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// 获取进程id
    pub fn pid(&self) -> usize {
        self.pid
    }

    /// 获取进程地址空间
    pub fn memory_set(&self) -> Arc<MemorySet> {
        self.memory_set.clone()
    }

    /// 获取根线程
    pub fn root_thread(&self) -> Arc<Thread> {
        self.threads.get().get(&0).unwrap().clone()
    }

    /// 删除子进程
    pub fn remove_child(&self, pid: usize) {
        let children = self.children.get_mut();
        for index in 0..children.len() {
            let child = children.get(index);
            if let Some(child) = child {
                if child.pid() == pid {
                    children.remove(index);
                    return;
                }
            }
        }
    }

    /// 删除一个线程
    pub fn remove_thread(&self, tid: usize) {
        self.threads.get_mut().remove(&tid);
    }

    /// 获得线程的引用
    pub fn get_thread(&self, tid: usize) -> Option<Arc<Thread>> {
        let thread = if let Some(thread) = self.threads.get().get(&tid) {
            Some(thread.clone())
        } else {
            None
        };
        thread
    }

    /// 获取父进程的引用
    pub fn parent(&self) -> Option<Arc<Process>> {
        self.parent.read().upgrade()
    }

    /// 获取文件表
    pub fn file_table(&self) -> &mut Vec<Option<Arc<dyn File>>> {
        self.file_table.get_mut()
    }

    /// 获取互斥锁列表
    pub fn mutexes(&self) -> &mut Vec<Arc<MutexBlocking>> {
        self.mutexes.get_mut()
    }

    /// 获取信号量列表
    pub fn sems(&self) -> &mut Vec<Arc<Sem>> {
        self.sems.get_mut()
    }

    /// 获取条件变量列表
    pub fn condvars(&self) -> &mut Vec<Arc<Condvar>> {
        self.condvars.get_mut()
    }
}
