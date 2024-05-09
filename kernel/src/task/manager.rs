//! 线程管理器

use super::*;
use crate::*;
use alloc::collections::VecDeque;

/// 线程管理器
#[derive(Default)]
pub struct ThreadManager {
    runnable: VecDeque<ThreadPtr>,
}

impl ThreadManager {
    /// 向队尾加入一个线程
    pub fn enqueue(&mut self, t: &mut Thread) {
        self.runnable.push_back(unsafe { transmute(t) });
    }

    /// 从队头弹出第一个就绪线程
    pub fn dequeue(&mut self) -> &'static mut Thread {
        self.runnable.pop_front().unwrap()
    }

    /// 从就绪线程队列中移除不能运行的线程
    pub fn clear_zombie(&mut self) {
        let len = self.runnable.len();
        for _ in 0..len {
            let t = self.runnable.pop_front().unwrap();
            if t.state == ThreadState::Runnable {
                self.runnable.push_back(t);
            }
        }
    }

    /// 调度，从就绪线程队列中弹出并运行下一个就绪线程
    pub fn resched(&mut self) {
        let cur = current();
        if cur.state == ThreadState::Runnable {
            self.enqueue(cur);
        }
        let nxt = self.dequeue();
        if cur as *const _ != nxt as *const _ {
            cur.switch_to(nxt);
        }
    }
}
