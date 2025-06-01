//! 进程
use super::*;
use hybrid_objects::{mm::MemorySet};
use alloc::string::String;
use alloc::sync::Weak;
use alloc::vec::Vec;
use thread::Tid;
use lazy_static::lazy_static;
use spin::RwLock;
use alloc::collections::BTreeMap;

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