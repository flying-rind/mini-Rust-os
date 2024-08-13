//! 用户线程

use core::task::Waker;

use crate::syscall::syscall;
use crate::*;
use alloc::sync::{Arc, Weak};
use mm::MemoryArea;
use trapframe::UserContext;
use x86_64::instructions::tlb;

/// 全局变量：当前线程
pub static CURRENT_THREAD: Cell<Option<Arc<Thread>>> = Cell::new(None);

/// 全局变量：用户线程队列
pub static THREAD_DEQUE: Cell<VecDeque<Arc<Thread>>> = Cell::new(VecDeque::new());

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
    /// 栈内存区域
    #[allow(unused)]
    stack_area: Arc<MemoryArea>,
    /// 状态改变时的唤醒器
    state_wakers: Cell<Vec<(Waker, ThreadState)>>,
}

impl Thread {
    /// 创建一个新的线程
    pub fn new(
        proc: Weak<Process>,
        tid: usize,
        entry: usize,
        sp: usize,
        arg1: usize,
        arg2: usize,
        stack_area: Arc<MemoryArea>,
    ) -> Arc<Self> {
        // 定义线程用户运行上下文
        let mut context = Box::new(UserContext::default());
        // 设置sp寄存器
        context.general.rsp = sp;
        // 设置ip寄存器
        context.general.rip = entry;
        context.general.rdi = arg1;
        context.general.rsi = arg2;
        context.general.rflags = 0x3000 | 0x200 | 0x2;

        // 创建线程
        let thread = Arc::new(Thread {
            proc,
            tid,
            state: Cell::new(ThreadState::Stop),
            user_context: Cell::new(context),
            stack_area,
            state_wakers: Cell::new(Vec::new()),
        });

        // 加入全局线程队列
        let thread_deque = THREAD_DEQUE.get_mut();
        thread_deque.push_back(thread.clone());
        thread
    }

    /// 运行当前线程，当用户态发生中断或系统调用时控制流返回Rust
    pub fn run_until_trap(&self) {
        // 切换当前线程所属进程的地址空间
        if let Some(proc) = self.proc.upgrade() {
            proc.memory_set().activate();
            // 刷新TLB
            tlb::flush_all();
        } else {
            panic!("[Kernel] Process already dropped");
        }

        self.user_context.get_mut().run()
    }

    /// 线程执行系统调用
    pub fn do_syscall(&self) {
        let syscall_num = self.user_context.get_syscall_num();
        let args = self.user_context.get_syscall_args();

        // 执行系统调用
        let (ret0, ret1) = syscall(syscall_num, args);
        self.user_context.get_mut().set_syscall_ret(ret0, ret1);
    }

    /// 线程退出，删除其所属进程中对其的引用
    ///
    /// 若为根线程，则退出相应的进程
    ///
    /// 这个函数应该被调度器执行
    pub fn exit(&self) {
        let process = self.proc.upgrade();
        if process.is_none() {
            // panic!("[Kernel] Proc already exited")
            // 进程已经被清理了，直接退出
            return;
        }
        let process = process.unwrap();
        if self.tid == 0 {
            // 退出进程
            process.exit();
        } else {
            // 删除进程对自己的引用
            process.remove_thread(self.tid);
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

    /// 获取用户栈
    pub fn stack_area(&self) -> Arc<MemoryArea> {
        self.stack_area.clone()
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

// impl Drop for Thread {
//     fn drop(&mut self) {
//         println!("[Rust] Thread dropped now: tid: {}", self.tid);
//     }
// }
