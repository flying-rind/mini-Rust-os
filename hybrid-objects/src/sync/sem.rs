//! 信号量
use crate::*;

use alloc::sync::Arc;
use core::task::Waker;
use future::{executor, futures::sync::WaitForSem};

/// 信号量
pub struct Sem {
    // 资源数量
    pub n: Cell<usize>,
    // 等待当前信号量资源的线程队列
    pub wait_queue: Cell<VecDeque<(Arc<Thread>, Waker)>>,
}

impl Sem {
    // 新建一个信号量，资源数量设置为n
    pub fn new(n: usize) -> Arc<Self> {
        Arc::new(Self {
            n: Cell::new(n as _),
            wait_queue: Cell::default(),
        })
    }

    // 增加一个资源
    pub fn up(&self) {
        *self.n.get_mut() += 1;
        // 若有线程阻塞等待，将阻塞队列中的第一个唤醒
        if let Some((_thread, waker)) = self.wait_queue.get_mut().pop_front() {
            // 唤醒等待协程
            waker.wake_by_ref();
        }
    }

    // 消耗一个资源
    pub fn down(&self, arc_self: Arc<Sem>) {
        if *self.n.get() >= 1 {
            *self.n.get_mut() -= 1;
        // 当前没有资源，阻塞当前线程，并生成协程
        } else {
            let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
            executor::spawn(WaitForSem::new(current_thread, arc_self))
        }
    }

    /// 阻塞队列中添加一个线程
    pub fn add_thread(&self, (thread, waker): (Arc<Thread>, Waker)) {
        self.wait_queue.get_mut().push_back((thread, waker));
    }
}
