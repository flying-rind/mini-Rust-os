//! 内核线程
use super::*;
use crate::alloc::string::ToString;
use crate::future::*;
use crate::kthread::processor_entry;
use crate::mm::*;
use crate::requests::*;
use alloc::{collections::VecDeque, string::String, sync::Arc};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::Waker;
use hashbrown::HashMap;
use requests::Processor;
use spin::Lazy;
use trap::CalleeRegs;

core::arch::global_asm!(include_str!("switch.S"));

extern "C" {
    /// 内嵌汇编，保存当前内核线程现场，并恢复下一个内核线程现场
    pub fn context_switch(cur: &KernelContext, nxt: &KernelContext);
    /// 内嵌汇编，直接恢复下一个内核线程现场而不保存当前现场
    pub fn restore_next(nxt: &KernelContext);
}

/// 内核线程ID，从1开始，0为内核主线程
pub static KTHREAD_ID: AtomicUsize = AtomicUsize::new(1);

/// 当前内核线程
pub static CURRENT_KTHREAD: Cell<Option<Arc<Kthread>>> = Cell::new(None);

/// 内核线程队列
pub static KTHREAD_DEQUE: Cell<VecDeque<Arc<Kthread>>> = Cell::new(VecDeque::new());

/// 内核线程服务类型到内核线程的映射
pub static KTHREAD_MAP: Lazy<Cell<HashMap<KthreadType, Arc<Kthread>>>> =
    Lazy::new(|| Cell::new(HashMap::new()));

/// 内核线程状态
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub enum KthreadState {
    #[default]
    /// 空闲
    Idle,
    /// 有请求等待处理，需要运行
    NeedRun,
}

/// 内核线程的服务类型
#[derive(Default, PartialEq, Eq, Hash, Clone)]
pub enum KthreadType {
    /// 文件系统服务
    FS,
    /// 块设备服务
    BLK,
    /// 执行器
    EXECUTOR,
    /// 根线程
    ROOT,
    /// 未指定
    #[default]
    UNKNOWN,
}

/// 内核线程的内核态现场
#[derive(Default)]
#[repr(C)]
pub struct KernelContext {
    /// 被调用者保存寄存器
    pub regs: CalleeRegs,
    /// 内核线程入口现场
    pub rip: usize,
}

/// 内核线程
///
/// 每个内核线程有独立的内核栈
#[derive(Default)]
pub struct Kthread {
    /// 内核线程ID
    ktid: usize,
    /// 内核线程名称
    name: String,
    /// 内核线程的内核态上下文
    context: Cell<Box<KernelContext>>,
    /// 运行状态
    state: Cell<KthreadState>,
    /// 服务类型
    #[allow(unused)]
    ktype: KthreadType,
    /// 用户请求的实际处理器
    processor: Option<Arc<dyn Processor>>,
    /// 请求队列
    request_queue: Cell<VecDeque<(Request, usize)>>,
    /// 请求的唤醒器队列
    request_wakers: Cell<Vec<(Waker, usize)>>,
    /// 最新的请求ID
    request_id: Cell<usize>,
    /// 已经响应的请求ID
    response_id: Cell<usize>,
    /// 当前正在处理的请求的ID
    current_request_id: Cell<usize>,
}

impl Kthread {
    /// 创建内核线程
    pub fn new(
        name: String,
        entry: usize,
        processor: Option<Arc<dyn Processor>>,
        ktype: KthreadType,
    ) -> Arc<Kthread> {
        let ktid = KTHREAD_ID.fetch_add(1, Ordering::Relaxed);

        // 两个线程栈之间空余一小段空间
        let stack_base = KERNEL_STACK_BASE + ktid * KERNEL_STACK_SIZE * 2;

        // 初始化内核现场
        let mut context = KernelContext::default();
        // 设置sp，ip
        context.regs.rsp = stack_base + KERNEL_STACK_SIZE;
        context.rip = entry;

        // 创建新内核线程
        let kthread = Arc::new(Kthread {
            ktid,
            name,
            context: Cell::new(Box::new(context)),
            processor,
            ktype,
            ..Kthread::default()
        });

        // 将内核线程放入全局线程队列
        KTHREAD_DEQUE.get_mut().push_back(kthread.clone());
        kthread
    }

    /// 切换到下一个内核线程
    pub fn switch_to(&self, next: Arc<Kthread>) {
        unsafe {
            context_switch(&(*(self.context.get())), &(*(next.context.get())));
        }
    }

    /// 切换下一个内核线程且不保存当前现场
    pub fn switch_to_without_saving_context(&self, next: Arc<Kthread>) {
        unsafe {
            restore_next(&(*(next.context.get())));
        }
    }

    /// 重启内核线程，当内核线程内发生严重错误panic时在panic handler中使用（测试）
    ///
    /// 首先唤醒出现错误的请求，重启后不再处理
    ///
    /// 重置当前内核线程的rip和rsp，并不保存上下文切换到其他内核线程执行
    pub fn reboot(&self, current_kthread: Arc<Kthread>) {
        // 重置上下文
        let current_req_id = self.current_request_id.get().clone();
        let context = self.context.get_mut();
        context.rip = processor_entry as usize;
        context.regs.rsp =
            KERNEL_STACK_BASE + self.ktid * KERNEL_STACK_SIZE * 2 + KERNEL_STACK_SIZE;
        // 唤醒出错的请求
        self.wake_request(current_req_id);
        let kthread = Scheduler::get_first_kthread().unwrap();
        KTHREAD_DEQUE.get_mut().push_back(current_kthread.clone());
        // 修改全局变量，且不保存寄存器
        *CURRENT_KTHREAD.get_mut() = Some(kthread.clone());
        println!(
            "\x1b[32mKthread: {} reboot success\x1b[0m",
            current_kthread.name()
        );
        current_kthread.switch_to_without_saving_context(kthread);
    }

    /// 获取自己的类型
    pub fn ktype(&self) -> &KthreadType {
        &self.ktype
    }

    /// 添加一个请求
    ///
    /// 返回请求的ID
    pub fn add_request(&self, request: Request) -> usize {
        let req_id = self.request_id.get().clone() + 1;
        self.request_queue.get_mut().push_back((request, req_id));
        *self.request_id.get_mut() += 1;
        // 接到请求立刻设置内核线程需要运行
        *self.state.get_mut() = KthreadState::NeedRun;
        return req_id;
    }

    /// 获取第一个请求
    ///
    /// 若为None表示请求队列为空
    pub fn get_first_request(&self) -> Option<(Request, usize)> {
        self.request_queue.get_mut().pop_front()
    }

    /// 添加一个唤醒器
    pub fn add_waker(&self, waker: Waker, requset_id: usize) {
        self.request_wakers.get_mut().push((waker, requset_id));
    }

    /// 完成一个请求后，尝试使用其唤醒器唤醒协程
    pub fn wake_request(&self, request_id: usize) {
        // 更新自己的已响应ID
        *(self.response_id.get_mut()) = request_id;
        // 有可能在协程轮讯前先运行了内核线程完成了服务，则无需唤醒了
        for (waker, id) in self.request_wakers.get() {
            if *id == request_id {
                waker.wake_by_ref();
                return;
            }
        }
        // panic!("No waker for request ID: {}", request_id);
    }

    /// 创建根内核线程
    pub fn new_root() -> Arc<Kthread> {
        let root_kthread = Arc::new(Kthread {
            ktid: 0,
            name: "Root".to_string(),
            ktype: KthreadType::ROOT,
            ..Kthread::default()
        });
        // 设置当前内核线程为根线程
        let _ = CURRENT_KTHREAD.get_mut().insert(root_kthread.clone());
        root_kthread
    }

    /// 是否需要调度执行
    pub fn need_schedule(&self) -> bool {
        // 主线程，总是需要运行
        if self.ktid == 0 {
            return true;
        }
        // 执行器线程
        if self.name.contains("Executor") {
            return executor::need_schedule();
        }

        // 服务线程
        assert!(self.processor.is_some());
        return !(*self.state.get() == KthreadState::Idle);
    }

    /// 设置内核线程状态
    pub fn set_state(&self, state: KthreadState) {
        *self.state.get_mut() = state
    }

    /// 设置当前正在处理的请求ID
    pub fn set_current_request_id(&self, request_id: usize) {
        *self.current_request_id.get_mut() = request_id;
    }

    /// 获取当前正在处理的请求ID
    pub fn current_request_id(&self) -> usize {
        self.current_request_id.get().clone()
    }

    /// 获取内核线程状态
    pub fn state(&self) -> KthreadState {
        self.state.get().clone()
    }

    /// 获取自己的已响应请求ID
    pub fn response_id(&self) -> usize {
        self.response_id.get().clone()
    }

    /// 获取请求处理器
    pub fn processor(&self) -> Option<Arc<dyn Processor>> {
        match self.processor.as_ref() {
            Some(processor) => Some(processor.clone()),
            None => None,
        }
    }

    /// 获取名字
    pub fn name(&self) -> &str {
        &self.name
    }
}
