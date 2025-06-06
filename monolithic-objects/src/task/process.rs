//! 进程
use super::*;
use hybrid_objects::{fs::File, mm::MemorySet};
use alloc::string::String;
use alloc::sync::Weak;
use alloc::vec::Vec;
use thread::Tid;
use lazy_static::lazy_static;
use spin::RwLock;
use alloc::collections::BTreeMap;
use rcore_fs::vfs::{FsError, INode};
use crate::debug;
use hybrid_objects::fs::ROOT_INODE;

/// process id type
#[derive(Clone, Default, Ord, PartialEq, PartialOrd, Eq)]
pub struct Pid(pub usize);
/// process group id type
pub type Pgid = i32;

impl Pid {
    pub const INIT: usize = 1;

    pub fn new() -> Self {
        Pid(0)
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
    /// Pathname is interpreted relative to the current working directory(CWD)
    const AT_FDCWD: usize = -100isize as usize;
    pub const FOLLOW_MAX_DEPTH: usize = 3;


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
        debug!(
            "lookup_inode_at: dirfd: {:?}, cwd: {:?}, path: {:?}, follow: {:?}",
            dirfd as isize, self.cwd, path, follow
        );
        let follow_max_depth = if follow {Self::FOLLOW_MAX_DEPTH} else {0};
        // 从当前工作目录寻找
        if dirfd == Self::AT_FDCWD {
            Ok(ROOT_INODE
                    .lookup(&self.cwd)?
                    .lookup_follow(path, follow_max_depth)?
                )
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
}