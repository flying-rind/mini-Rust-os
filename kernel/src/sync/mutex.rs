//! 实现互斥锁
use crate::{task::*, *};

pub trait Mutex {
    fn lock(&self);
    fn unlock(&self);
}

/// 基于主动调度实现的互斥锁
#[derive(Default)]
pub struct MutexSpin {
    locked: Cell<bool>,
}

impl Mutex for MutexSpin {
    fn lock(&self) {
        loop {
            // 此时锁被占用，当前线程放弃CPU
            if *self.locked {
                task::current_yield();
            } else {
                *self.locked.get() = true;
                return;
            }
        }
    }

    fn unlock(&self) {
        *self.locked.get() = false;
    }
}

/// 基于阻塞和唤醒机制的互斥锁
#[derive(Default)]
pub struct MutexBlocking {
    locked: Cell<bool>,
    // 等待锁的线程队列
    wait_queue: Cell<VecDeque<ThreadPtr>>,
}

impl Mutex for MutexBlocking {
    fn lock(&self) {
        if *self.locked {
            self.wait_queue.get().push_back(task::current());
            task::sched_block();
        } else {
            *self.locked.get() = true;
        }
    }

    // 将自己等待队列中的第一个线程设置为就绪
    fn unlock(&self) {
        if let Some(t) = self.wait_queue.get().pop_front() {
            task::sched_unblock(t);
        } else {
            *self.locked.get() = false;
        }
    }
}
