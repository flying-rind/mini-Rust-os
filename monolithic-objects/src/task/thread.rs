//! 宏内核线程
use core::pin::Pin;

use super::*;
use trapframe::UserContext;
use spin::Mutex;
use alloc::boxed::Box;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use spin::RwLock;
use alloc::collections::BTreeMap;

lazy_static! {
    /// Records the mapping between pid and Process struct.
    pub static ref THREADS: RwLock<BTreeMap<usize, Arc<Thread>>> =
        RwLock::new(BTreeMap::new());
}

/// 全局变量：当前用户线程
pub static CURRENT_THREAD: Mutex<Option<Arc<Thread>>> = Mutex::new(None);

/// 设置当前用户线程
pub fn set_current_thread(thread: Option<Arc<Thread>>) {
    let mut cur_thread = CURRENT_THREAD.lock();
    *cur_thread = thread;
}

/// Tid type
pub type Tid = usize;

/// 用户线程的线程异步函数
pub type ThreadFn = fn(thread: Arc<Thread>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// 线程可变部分
#[derive(Default)]
pub struct ThreadInner {
    /// 用户态上下文
    pub context: Option<Box<UserContext>>,
}

/// 线程状态
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ThreadState {
    /// 已退出
    #[default]
    Exited = 0,
}

/// 线程
#[derive(Default)]
pub struct Thread {
    /// 可变部分
    pub inner: Mutex<ThreadInner>,
    /// Thread id
    pub tid: Tid,
    /// 所属进程
    pub proc: Arc<Mutex<Process>>,
    /// 线程状态
    pub state: ThreadState,
}

impl Thread {
    /// 分配一个tid并加入全局线程映射表
    pub fn add_to_table(mut self) -> Arc<Self> {
        let mut thread_table = THREADS.write();
        let tid = (1..).find(|i| thread_table.get(i).is_none()).unwrap();
        self.tid = tid;
        let self_ref = Arc::new(self);
        thread_table.insert(tid, self_ref.clone());
        self_ref
    }

    /// 从当前进程复制进程
    pub fn fork(&self, context: &UserContext) -> Arc<Thread> {
        // 复制进程地址空间
        let vm = self.proc.lock().vm.clone_myself();
        // 设置上下文
        let mut context = context.clone();
        context.set_syscall_ret(0, 0);
        // 创建子进程
        let mut cur_proc = self.proc.lock();
        let new_proc = Arc::new(Mutex::new(Process {
            vm,
            pid: Pid::new(),
            pgid: cur_proc.pgid,
            exit_code: 0,
            exec_path: cur_proc.exec_path.clone(),
            cwd: cur_proc.cwd.clone(),
            parent: (cur_proc.pid.clone(), Arc::downgrade(&self.proc)),
            ..Process::default()
        }));
        // 创建子进程主线程
        let new_thread = Thread {
            tid: 0,
            inner: Mutex::new(ThreadInner { context: Some(Box::new(context)) }),
            proc: new_proc.clone(),
            ..Thread::default()
        }.add_to_table();
        // 关联线程和进程，新进程的pid设置为新线程的tid
        let child_pid = Pid(new_thread.tid);
        add_to_process_table(new_proc.clone(), child_pid.clone());
        new_thread.proc.lock().threads.push(new_thread.tid);
        // 设置父进程
        cur_proc.children.push((child_pid, Arc::downgrade(&new_proc)));
        new_thread
    }
}

