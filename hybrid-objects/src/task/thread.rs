//! 用户线程

use core::task::Waker;

use crate::future::ThreadSwitchFuture;
use crate::mm::MemorySet;
use crate::*;
use alloc::sync::{Arc, Weak};
use core::pin::Pin;
use mm::MemoryArea;
use trapframe::UserContext;
use x86_64::structures::paging::PageTableFlags;

/// 用户线程的线程异步函数
pub type ThreadFn = fn(thread: Arc<Thread>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// 全局变量：当前线程
pub static CURRENT_THREAD: Cell<Option<Arc<Thread>>> = Cell::new(None);

/// 全局变量：用户线程队列
pub static THREADS_DEQUE: Cell<VecDeque<Arc<Thread>>> = Cell::new(VecDeque::new());

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum ThreadState {
    /// 可运行
    Runnable,
    /// 暂停
    #[default]
    Stop,
    /// 异步等待
    Waiting,
    /// 被调度
    Suspended,
    /// 已退出, 可以被回收
    Exited,
}

/// 用户线程
///
/// 用户线程共享内核线程栈
pub struct Thread {
    /// 线程id
    tid: usize,
    /// 线程所属进程
    proc: Weak<Process>,
    /// 线程状态
    state: Cell<ThreadState>,
    /// 线程执行的用户态上下文
    user_context: Cell<Box<UserContext>>,
    /// 状态改变时的唤醒器
    state_wakers: Cell<Vec<(Waker, ThreadState)>>,
}

impl Thread {
    /// 创建一个新的线程
    pub fn new(proc: Weak<Process>, tid: usize, entry: usize, sp: usize) -> Arc<Self> {
        // 定义线程用户运行上下文
        let mut context = Box::new(UserContext::default());
        // 设置sp寄存器
        context.general.rsp = sp;
        // 设置ip寄存器
        context.general.rip = entry;
        context.general.rflags = 0x3000 | 0x200 | 0x2;

        // 创建线程
        let thread = Arc::new(Thread {
            proc,
            tid,
            state: Cell::new(ThreadState::Stop),
            user_context: Cell::new(context),
            state_wakers: Cell::new(Vec::new()),
        });

        // 加入全局线程队列
        let thread_deque = THREADS_DEQUE.get_mut();
        thread_deque.push_back(thread.clone());
        thread
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
        use mm::{USER_STACK_BASE, USER_STACK_SIZE};
        let init_info = ProcInfo { args, envs, auxv };
        unsafe { init_info.push_at(USER_STACK_BASE + USER_STACK_SIZE) }
    }

    /// 运行当前线程，当用户态发生中断或系统调用时控制流返回Rust
    pub fn run_until_trap(&self) {
        self.user_context.get_mut().run()
    }

    /// Start execution on the thread. Add to executor
    pub fn start(self: Arc<Self>, thread_fn: ThreadFn) {
        let future = thread_fn(self.clone());
        let switch = ThreadSwitchFuture::new(self, future);
        executor::spawn(switch);
    }

    /// 设置用户态上下文的返回值
    pub fn set_syscall_ret(&self, ret0: usize, ret1: usize) {
        self.user_context.get_mut().set_syscall_ret(ret0, ret1);
    }

    /// 线程退出，删除其所属进程中对其的引用
    ///
    /// 若为根线程，则退出相应的进程
    pub fn exit(&self, exit_code: usize) {
        let process = self.proc.upgrade().unwrap();
        // 删除进程对自己的引用
        process.remove_thread(self.tid);
        // 删除全局线程表的引用
        THREADS_DEQUE.get_mut().remove(self.tid);
        self.set_state(ThreadState::Exited);
        if self.tid == 0 {
            // 退出进程
            process.exit(exit_code);
        }
    }

    /// 获取用户态上下文
    pub fn user_context(&self) -> &mut UserContext {
        &mut *(self.user_context.get_mut())
    }

    /// 设置用户态上下文
    pub fn set_user_context(&self, user_context: &mut UserContext) {
        **self.user_context.get_mut() = user_context.clone();
    }

    /// 设置ip
    pub fn set_ip(&self, ip: usize) {
        self.user_context.get_mut().general.rip = ip;
    }

    /// 设置sp
    pub fn set_sp(&self, sp: usize) {
        self.user_context.get_mut().general.rsp = sp;
    }

    /// 设置rdi，rsi
    pub fn set_args(&self, rdi: usize, rsi: usize) {
        self.user_context.get_mut().general.rdi = rdi;
        self.user_context.get_mut().general.rsi = rsi;
    }

    /// 设置rax
    pub fn set_rax(&self, rax: usize) {
        self.user_context.get_mut().general.rax = rax;
    }

    /// 添加一个状态唤醒器
    pub fn add_state_waker(&self, waker: Waker, state: ThreadState) {
        self.state_wakers.get_mut().push((waker, state));
    }

    /// 设置线程状态
    pub fn set_state(&self, new_state: ThreadState) {
        // 线程已经退出，不再改变状态
        if *self.state.get() == ThreadState::Exited {
            return;
        }
        *self.state.get_mut() = new_state;
        // 唤醒等待的唤醒器
        self.state_wakers.get_mut().retain(|state_waker| {
            let (waker, wait_state) = state_waker;
            if *wait_state == new_state {
                waker.wake_by_ref();
                return false;
            }
            true
        });
    }

    /// 从停止状态恢复
    pub fn resume(&self) {
        *self.state.get_mut() = ThreadState::Runnable;
    }

    /// 是否是根线程
    pub fn is_root(&self) -> bool {
        self.tid == 0
    }

    /// 获取tid
    pub fn tid(&self) -> usize {
        self.tid
    }

    /// 获取所属进程
    pub fn proc(&self) -> Option<Arc<Process>> {
        self.proc.upgrade()
    }

    /// 获取状态
    pub fn state(&self) -> ThreadState {
        self.state.get().clone()
    }
}
