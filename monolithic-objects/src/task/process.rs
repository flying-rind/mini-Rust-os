//! 进程
use core::fmt::Display;

use super::*;
use crate::debug;
use crate::task::abi::ProcInfo;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Weak;
use alloc::vec::Vec;
use hybrid_objects::fs::ROOT_INODE;
use hybrid_objects::mm::{USER_STACK_BASE, USER_STACK_SIZE, load_app};
use hybrid_objects::{fs::File, mm::MemorySet};
use lazy_static::lazy_static;
use log::info;
use rcore_fs::vfs::{FsError, INode};
use spin::RwLock;
use thread::{Thread, Tid};
use trapframe::UserContext;
use xmas_elf::ElfFile;

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
    pub files: BTreeMap<usize, Arc<dyn File>>,
    /// Parent process
    pub parent: (Pid, Weak<Mutex<Process>>),
    /// Children process
    pub children: Vec<(Pid, Weak<Mutex<Process>>)>,
    /// Threads
    pub threads: Vec<Tid>,
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
    pub fn get_file(&mut self, fd: usize) -> Result<Arc<dyn File>, usize> {
        const EBADF: usize = 9;
        self.files.get_mut(&fd).ok_or(EBADF).cloned()
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
    ) -> Result<usize, FsError> {
        // Read ELF header
        // 0x3c0: magic number from ld-musl.so
        let mut data = [0u8; 16 * 1024 * 1024];
        inode.read_at(0, &mut data)?;

        // paese elf
        let elf = ElfFile::new(&data).map_err(|_| FsError::NotFile)?;
        let entry = elf.header.pt2.entry_point() as usize;
        // clear old elf, load new one.
        self.vm.clear_elf();
        load_app(self.vm.clone(), &elf);

        // Kill other threads
        self.threads.retain(|&tid| tid == cur_thread.tid);
        // 环境变量和参数压栈
        self.vm.activate();
        let init_info = ProcInfo { args, envs };
        let sp = unsafe { init_info.push_at(USER_STACK_BASE + USER_STACK_SIZE) };
        // 修改线程上下文
        context.set_ip(entry);
        context.set_sp(sp);
        Ok(0)
    }

    /// Exit the process
    pub fn exit(&mut self, exit_code: usize) {
        // Clear fd_table
        self.files.clear();
        info!("Process {} exit with {}", self.pid.get(), exit_code);
    }

    /// Check if is exied
    pub fn exited(&self) -> bool {
        self.threads.is_empty()
    }
}
