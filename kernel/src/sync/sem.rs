//! 信号量
use crate::{task::*, *};

/// 信号量
pub struct Sem {
    // 资源数量
    pub n: Cell<isize>,
    // 等待当前信号量资源的线程队列
    pub wait_queue: Cell<VecDeque<ThreadPtr>>,
}

impl Sem {
    // 新建一个信号量，资源数量设置为n
    pub fn new(n: usize) -> Self {
        Self {
            n: Cell::new(n as _),
            wait_queue: Cell::default(),
        }
    }

    // 增加一个资源
    pub fn up(&self) {
        *self.n.get() += 1;
        if *self.n <= 0 {
            // 若有线程阻塞等待，将阻塞队列中的第一个唤醒
            if let Some(t) = self.wait_queue.get().pop_front() {
                task::sched_unblock(t);
            }
        }
    }

    // 消耗一个资源
    pub fn down(&self) {
        *self.n.get() -= 1;
        if *self.n < 0 {
            self.wait_queue.get().push_back(task::current());
            // 阻塞当前线程
            task::sched_block();
        }
    }
}
