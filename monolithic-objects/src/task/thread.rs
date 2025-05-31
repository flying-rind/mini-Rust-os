//! 宏内核线程
use core::default;

use super::*;
use hybrid_objects::vec::Vec;
use spin::mutex::Mutex;
use trapframe::UserContext;
use spin::{Mutex, RwLock};
use alloc::boxed::Box;
use alloc::sync::Arc;


/// 线程可变部分
#[derive(Default)]
pub struct ThreadInner {
    /// 用户态上下文
    context: Box<UserContext>,
}

/// Tid type
pub type Tid = usize;

lazy_static! {
    /// Records the mapping between pid and Process struct.
    pub static ref THREADS: RwLock<BTreeMap<usize, Arc<Thread>>> =
        RwLock::new(BTreeMap::new());
}

/// 线程
#[derive(Default)]
pub struct Thread {
    /// 可变部分
    inner: Mutex<ThreadInner>,
    /// Thread id
    tid: Tid,
    /// 所属进程
    proc: Arc<Mutex<Process>>
}

impl Thread {
    /// 分配一个tid并加入全局线程映射表
    pub fn add_to_table(&self) -> Arc<Self> {
        let mut thread_table = THREADS.write();
        let tid = (1..).find(|i| thread_table.get(i).is_none()).unwrap();
        self.tid = tid;

        thread_table.insert(tid, Arc::new(self));
        Arc::new(self)
    }

    /// 从当前进程复制进程
    pub fn fork(&self, context: &UserContext) -> Arc<Thread> {
        // 复制进程地址空间
        let vm = self.proc.lock().vm.clone_myself();
        // 设置上下文
        let mut context = context.clone();
        context.set_syscall_ret(0, 0);
        // 创建子进程
        let cur_proc = self.proc.lock();
        let new_proc = Arc::new(Mutex::new(Process {
            vm,
            pid: Pid::new(),
            pgid: cur_proc.pgid,
            exit_code: 0,
            exec_path: cur_proc.exec_path.clone(),
            cwd: cur_proc.cwd.clone(),
            parent: (cur_proc.pid.clone(), Arc::downgrade(&self.proc)),
            children: Vec::new(),
            threads: Vec::new(),
        }));
        // 创建子进程主线程
        let new_thread = Thread {
            tid: 0,
            inner: Mutex::new(ThreadInner { context: Box::new(context) }),
            proc: new_proc,
        }.add_to_table();
        // 关联线程和进程，新进程的pid设置为新线程的tid
        let child_pid = Pid(new_thread.tid);
        add_to_process_table(new_proc, child_pid);
        new_thread.proc.lock().threads.push(new_thread.tid);
        // 设置父进程
        cur_proc.children.push((child_pid, Arc::downgrade(&new_proc)));
        new_thread
    }
}