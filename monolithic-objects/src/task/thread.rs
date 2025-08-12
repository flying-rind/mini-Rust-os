//! 宏内核线程
use super::*;
use crate::Siginfo;
use crate::Signal;
use crate::SignalStack;
use crate::Sigset;
use crate::fs::file::File;
use crate::sync::EventBus;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::sync::Weak;
use alloc::vec::Vec;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use hal::SysError;
use hybrid_objects::fs::Stdin;
use hybrid_objects::fs::Stdout;
use hybrid_objects::mm::MemoryArea;
use hybrid_objects::mm::MemorySet;
use lazy_static::lazy_static;
use log::info;
use num::FromPrimitive;
use rcore_fs::vfs::INode;
use spin::Mutex;
use spin::RwLock;
use trapframe::UserContext;
use x86_64::structures::paging::PageTableFlags;

lazy_static! {
    /// Records the mapping between tid and Thread struct.
    pub static ref THREADS: RwLock<BTreeMap<Tid, Arc<Thread>>> =
        RwLock::new(BTreeMap::new());
}

/// 全局变量：当前用户线程
pub static CURRENT_THREAD: Mutex<Option<Arc<Thread>>> = Mutex::new(None);

/// 设置当前用户线程
pub fn set_current_thread(thread: Option<Arc<Thread>>) {
    let mut cur_thread = CURRENT_THREAD.lock();
    *cur_thread = thread;
}

/// Get thread by tid.
pub fn get_thread(tid: usize) -> Option<Arc<Thread>> {
    let threads = THREADS.read();
    threads.get(&tid).cloned()
}

/// Tid type
pub type Tid = usize;

/// 用户线程的线程异步函数
pub type ThreadFn = fn(thread: Arc<Thread>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// 线程可变部分
#[derive(Default, Clone)]
pub struct ThreadInner {
    /// 用户态上下文
    pub context: Option<Box<UserContext>>,
    /// Kernel performs futex wake when thread exits.
    /// Ref: [http://man7.org/linux/man-pages/man2/set_tid_address.2.html]
    pub clear_child_tid: usize,
    /// 线程状态
    pub state: ThreadState,
    /// Signal mask.
    pub signal_mask: Sigset,
    /// handling signals.
    pub handling_signal: Option<Signal>,
    /// Signal alternate stack.
    pub signal_altstack: SignalStack,
}

/// 线程状态
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ThreadState {
    /// 已退出
    #[default]
    Exited = 0,
    /// 可运行
    Ready = 1,
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
}

impl Thread {
    /// Take away the context, then begin running.
    pub fn begin_running(&self) -> Box<UserContext> {
        self.inner.lock().context.take().unwrap()
    }

    /// Put back the context
    pub fn end_running(&self, ctx: Box<UserContext>) {
        self.inner.lock().context = Some(ctx)
    }

    /// 分配一个tid并加入全局线程映射表
    pub fn add_to_table(mut self) -> Arc<Self> {
        let mut thread_table = THREADS.write();
        let tid = (1..).find(|i| thread_table.get(i).is_none()).unwrap();
        self.tid = tid;
        let self_ref = Arc::new(self);
        thread_table.insert(tid, self_ref.clone());
        self_ref
    }

    /// Set user context, set ip and sp
    pub fn set_context(&self, ip: usize, sp: usize) {
        let mut inner = self.inner.lock();
        let context = inner.context.as_mut().unwrap();
        context.set_ip(ip);
        context.set_sp(sp);
    }

    /// Construct a new user stack memory area, insert to vm.
    /// And push args and envs to stack, return new sp.
    pub fn new_user_stack(
        vm: Arc<MemorySet>,
        args: Vec<String>,
        envs: Vec<String>,
        auxv: BTreeMap<u8, usize>,
    ) -> usize {
        let flags =
            PageTableFlags::WRITABLE | PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
        let stack_area = MemoryArea::new(USER_STACK_BASE, USER_STACK_SIZE, flags);
        vm.insert_area(stack_area);
        // 参数压栈
        vm.activate();
        use hal::abi::ProcInfo;
        use hybrid_objects::mm::{USER_STACK_BASE, USER_STACK_SIZE};
        let init_info = ProcInfo { args, envs, auxv };
        unsafe { init_info.push_at(USER_STACK_BASE + USER_STACK_SIZE) }
    }

    /// Construct a new user process, should only be used in root process.
    pub fn new_user(
        inode: &Arc<dyn INode>,
        exec_path: &str,
        args: Vec<String>,
        envs: Vec<String>,
    ) -> Result<Arc<Thread>, SysError> {
        let (new_vm, entry, sp) = Process::new_user_vm(inode, args, envs)?;
        // 构造用户上下文和用户线程
        let mut context = UserContext::default();
        context.set_ip(entry);
        context.set_sp(sp);
        let mut files: BTreeMap<usize, File> = BTreeMap::new();
        files.insert(0, File::Stdin(Stdin));
        files.insert(1, File::Stdout(Stdout));
        files.insert(2, File::Stdout(Stdout));
        let thread = Thread {
            inner: Mutex::new(ThreadInner {
                clear_child_tid: 0,
                state: ThreadState::Ready,
                context: Some(Box::new(context)),
                ..Default::default()
            }),
            proc: Arc::new(Mutex::new(Process {
                pid: Pid::new(),
                pgid: 0,
                exit_code: 0,
                vm: new_vm,
                exec_path: String::from(exec_path),
                cwd: String::from("/"),
                files: files,
                futexes: BTreeMap::new(),
                parent: (Pid::new(), Weak::new()),
                children: Vec::new(),
                threads: Vec::new(),
                eventbus: Arc::new(Mutex::new(EventBus::default())),
                ..Default::default()
            })),
            tid: 0,
        };
        let res = thread.add_to_table();
        Ok(res)
    }

    /// 从当前进程复制进程
    pub fn fork(&self, context: &mut Box<UserContext>) -> Arc<Thread> {
        // 复制进程地址空间
        let vm = self.proc.lock().vm.clone_myself();
        // 设置上下文
        let mut new_context = *context.clone();
        new_context.set_syscall_ret(0, 0);
        // 创建子进程
        let mut cur_proc = self.proc.lock();
        let new_proc = Arc::new(Mutex::new(Process {
            vm,
            pid: Pid::new(),
            pgid: cur_proc.pgid,
            exit_code: 0,
            exec_path: cur_proc.exec_path.clone(),
            cwd: cur_proc.cwd.clone(),
            files: cur_proc.files.clone(),
            parent: (cur_proc.pid.clone(), Arc::downgrade(&self.proc)),
            ..Process::default()
        }));
        // 创建子进程主线程
        let new_thread = Thread {
            tid: 0,
            inner: Mutex::new(ThreadInner {
                clear_child_tid: 0,
                context: Some(Box::new(new_context)),
                state: ThreadState::Ready,
                ..Default::default()
            }),
            proc: new_proc.clone(),
            ..Thread::default()
        }
        .add_to_table();
        // info!("Created new thread, tid = {}", new_thread.tid);
        // 关联线程和进程，新进程的pid设置为新线程的tid
        let child_pid = Pid(new_thread.tid);
        add_to_process_table(new_proc.clone(), child_pid.clone());
        new_thread.proc.lock().threads.push(new_thread.tid);
        // 设置父进程
        cur_proc
            .children
            .push((child_pid, Arc::downgrade(&new_proc)));
        new_thread
    }

    /// Start execution on the thread. Add to executor.
    pub fn start(self: Arc<Self>, thread_fn: ThreadFn) {
        let future = thread_fn(self.clone());
        let switch = ThreadSwitchFuture::new(self, future);
        executor::spawn(switch);
    }

    /// Exit the thread.
    pub fn exit(&self, exit_code: usize) {
        let tid = self.tid;
        info!("Thread exit, tid: {}, code: {}", tid, exit_code);

        // Delete tid ref in process.
        let mut proc = self.proc.lock();
        proc.threads.retain(|&id| id != tid);

        // Delete arc ref in THREAD table;
        let mut threads_table = THREADS.write();
        threads_table.remove(&tid);

        // for last thread, eixt the process
        if proc.threads.len() == 0 {
            proc.exit(exit_code);
        };

        // Perform futex wake 1.
        // ref: http://man7.org/linux/man-pages/man2/set_tid_address.2.html
        let mut inner = self.inner.lock();
        inner.state = ThreadState::Exited;
        let clear_child_tid = inner.clear_child_tid as *mut u32;
        if !clear_child_tid.is_null() {
            let futex = proc.get_futex(clear_child_tid as usize);
            info!("Perform futex {:#?} wake 1", clear_child_tid);
            // FIXME: Check first.
            unsafe {
                *clear_child_tid = 0;
                futex.wake(1);
            }
        }
    }

    /// Handle signal.
    /// Return (idx, info, sigmask).
    pub fn handle_signal(&self) -> Option<(usize, Siginfo, Sigset)> {
        let mut inner = self.inner.lock();
        if inner.handling_signal.is_none() {
            let mut proc = self.proc.lock();
            // clone the signals form process.
            let signals = proc.signals.clone();
            if let Some((idx, (info, _tid))) = {
                signals.iter().enumerate().find(|(_idx, (info, tid))| {
                    (*tid as usize == self.tid || *tid == -1)
                        && (!inner
                            .signal_mask
                            .contains(FromPrimitive::from_i32(info.signo).unwrap()))
                })
            } {
                inner.handling_signal = FromPrimitive::from_i32(info.signo);
                let signal: Signal = FromPrimitive::from_i32(info.signo).unwrap();
                proc.remove_signal(idx);
                proc.remove_pendingsigset(signal);
                return Some((idx, *info, inner.signal_mask));
            }
        }
        None
    }
}

type ThreadFuture = dyn Future<Output = ()> + Send;
type ThreadFuturePinned = Pin<Box<ThreadFuture>>;

/// Top level future, directly polled by the executor.
///
/// Make sure every time poll this future, modify the CURRENT_THREAD.
pub struct ThreadSwitchFuture {
    thread: Arc<Thread>,
    future: Mutex<ThreadFuturePinned>,
}

impl ThreadSwitchFuture {
    /// Spawn a new thread that can be polled by executor.
    pub fn new(thread: Arc<Thread>, future: ThreadFuturePinned) -> Self {
        Self {
            thread,
            future: Mutex::new(future),
        }
    }
}

impl Future for ThreadSwitchFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Switch vm.
        self.thread.proc.lock().vm.activate();
        set_current_thread(Some(self.thread.clone()));
        // Poll thread fn.
        let ret = self.future.lock().as_mut().poll(cx);
        set_current_thread(None);
        ret
    }
}
