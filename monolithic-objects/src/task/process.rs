//! 进程
use core::fmt::Display;

use super::*;
use crate::debug;
use crate::fs::file::File;
use crate::sync::{Event, EventBus, Futex};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Weak;
use alloc::vec;
use alloc::vec::Vec;
use hal::ElfExt;
use hal::PAGE_SIZE;
use hal::SysError;
use hal::SysError::EBADF;
use hal::abi;
use hybrid_objects::fs::ROOT_INODE;
use hybrid_objects::mm::MemorySet;
use hybrid_objects::mm::load_app;
use lazy_static::lazy_static;
use log::info;
use rcore_fs::vfs::{FsError, INode};
use spin::RwLock;
use thread::{Thread, Tid};
use trapframe::UserContext;
use xmas_elf::ElfFile;
use xmas_elf::header;

/// process id type
#[derive(Clone, Default, Ord, PartialEq, PartialOrd, Eq, Copy)]
pub struct Pid(pub usize);
impl Display for Pid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// process group id type
pub type Pgid = i32;

impl Pid {
    pub const INIT: usize = 1;

    /// Return 0
    pub fn new() -> Self {
        Pid(0)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

/// 宏内核进程
#[derive(Default)]
pub struct Process {
    /// Pid
    pub pid: Pid,
    /// Process group id
    pub pgid: Pgid,
    /// exit code
    pub exit_code: usize,
    /// 地址空间
    pub vm: Arc<MemorySet>,
    /// Executable path
    pub exec_path: String,
    /// 当前工作目录
    pub cwd: String,
    /// 文件表
    pub files: BTreeMap<usize, File>,
    /// Futex table.
    pub futexes: BTreeMap<usize, Arc<Futex>>,
    /// Parent process
    pub parent: (Pid, Weak<Mutex<Process>>),
    /// Children process
    pub children: Vec<(Pid, Weak<Mutex<Process>>)>,
    /// Threads
    pub threads: Vec<Tid>,
    /// Event bus
    pub eventbus: Arc<Mutex<EventBus>>,
}

lazy_static! {
    /// Records the mapping between pid and Process struct.
    pub static ref PROCESSES: RwLock<BTreeMap<Pid, Arc<Mutex<Process>>>> =
        RwLock::new(BTreeMap::new());
}

/// 设置pid并加入全局进程映射表
pub fn add_to_process_table(proc: Arc<Mutex<Process>>, pid: Pid) {
    let mut process_table = PROCESSES.write();

    // set pid
    proc.lock().pid = pid.clone();

    // put to process table
    process_table.insert(pid, proc.clone());
}

impl Process {
    const AT_FDCWD: usize = -100isize as usize;

    /// Lookup Inode from the process.
    ///
    /// - If `path` is relative, then it is interpreted relative to the directory
    ///   referred to by the file descriptor `dirfd`.
    ///
    /// - If the `dirfd` is the special value `AT_FDCWD`, then the directory is
    ///   current working directory of the process.
    ///
    /// - If `path` is absolute, then `dirfd` is ignored.
    ///
    /// - If `follow` is true, then dereference `path` if it is a symbolic link.
    pub fn lookup_inode_at(
        &self,
        dirfd: usize,
        path: &str,
        follow: bool,
    ) -> Result<Arc<dyn INode>, FsError> {
        /// Pathname is interpreted relative to the current working directory(CWD)
        pub const FOLLOW_MAX_DEPTH: usize = 3;
        debug!(
            "lookup_inode_at: dirfd: {:?}, cwd: {:?}, path: {:?}, follow: {:?}",
            dirfd as isize, self.cwd, path, follow
        );
        let follow_max_depth = if follow { FOLLOW_MAX_DEPTH } else { 0 };
        // 从当前工作目录寻找
        if dirfd == Self::AT_FDCWD {
            Ok(ROOT_INODE
                .lookup(&self.cwd)?
                .lookup_follow(path, follow_max_depth)?)
        // 从进程文件表中的dir_fd查找
        } else {
            let file = self.files.get(&dirfd).ok_or(FsError::EntryNotFound)?;
            Ok(file.lookup_follow(path, follow_max_depth)?)
        }
    }

    /// 在进程当前目录查找INode
    pub fn lookup_inode(&self, path: &str) -> Result<Arc<dyn INode>, FsError> {
        self.lookup_inode_at(Self::AT_FDCWD, path, true)
    }

    /// Get file
    pub fn get_file(&mut self, fd: usize) -> Result<&mut File, SysError> {
        self.files.get_mut(&fd).ok_or(EBADF)
    }

    /// Get lowest free fd.
    pub fn get_free_fd(&self) -> usize {
        (0..).find(|i| !self.files.contains_key(i)).unwrap()
    }

    /// Get the lowest available fd greated or equal to arg.
    pub fn get_free_fd_from(&self, arg: usize) -> usize {
        (arg..).find(|i| !self.files.contains_key(i)).unwrap()
    }

    /// Get futex by addr.
    pub fn get_futex(&mut self, uaddr: usize) -> Arc<Futex> {
        if !self.futexes.contains_key(&uaddr) {
            self.futexes.insert(uaddr, Arc::new(Futex::new()));
        }
        self.futexes.get(&uaddr).unwrap().clone()
    }

    /// Construct virtual memory of a new user process from ELF at `inode`.
    /// Return `(MemorySet, entry_point, ustack_top)`
    pub fn new_user_vm(
        inode: &Arc<dyn INode>,
        args: Vec<String>,
        envs: Vec<String>,
    ) -> Result<(Arc<MemorySet>, usize, usize), SysError> {
        let vm = MemorySet::new();
        // Read ELF header
        // 0x3c0: magic number from ld-musl.so
        let size = inode.metadata().unwrap().size;
        let mut buf = vec![0u8; size];
        inode.read_at(0, buf.as_mut_slice())?;

        // Parse ELF
        let elf = ElfFile::new(&buf).map_err(|_| SysError::EINVAL)?;

        // Check ELF type
        match elf.header.pt2.type_().as_type() {
            header::Type::Executable => {}
            header::Type::SharedObject => {}
            _ => return Err(SysError::EPERM),
        }
        load_app(vm.clone(), &elf);
        // auxiliary vector
        let auxv = {
            let mut map = BTreeMap::new();
            if let Some(phdr_vaddr) = elf.get_phdr_vaddr() {
                map.insert(abi::AT_PHDR, phdr_vaddr as usize);
            }
            map.insert(abi::AT_PHENT, elf.header.pt2.ph_entry_size() as usize);
            map.insert(abi::AT_PHNUM, elf.header.pt2.ph_count() as usize);
            map.insert(abi::AT_PAGESZ, PAGE_SIZE);
            map
        };

        // entry point
        let entry_addr = elf.header.pt2.entry_point() as usize;
        let sp = Thread::new_user_stack(vm.clone(), args, envs, auxv);
        return Ok((vm, entry_addr, sp));
    }

    /// 替换当前进程的elf文件
    /// FIXME: 适配MUSL
    /// return (ip, sp)
    pub fn exec(
        &mut self,
        inode: &Arc<dyn INode>,
        cur_thread: Arc<Thread>,
        args: Vec<String>,
        envs: Vec<String>,
        context: &mut Box<UserContext>,
    ) -> Result<usize, SysError> {
        let (new_vm, entry, sp) = Process::new_user_vm(inode, args, envs)?;
        self.vm = new_vm.clone();
        // Kill other threads
        self.threads.retain(|&tid| tid == cur_thread.tid);
        // 修改线程上下文
        **context = UserContext::default();
        context.set_ip(entry);
        context.set_sp(sp);
        Ok(0)
    }

    /// Exit the process
    pub fn exit(&mut self, exit_code: usize) {
        info!("Proc exit, pid: {}, code: {}", self.pid, exit_code);
        // Clear fd_table
        self.files.clear();
        info!("Process {} exit with {}", self.pid.get(), exit_code);

        // Set event bus.
        self.eventbus.lock().set(Event::PROCESS_QUIT);
        if let Some(parent) = self.parent.1.upgrade() {
            parent.lock().eventbus.lock().set(Event::CHILD_PROCESS_QUIT);
        }
        self.exit_code = exit_code;
    }

    /// Check if is exied
    pub fn exited(&self) -> bool {
        self.threads.is_empty()
    }
}
