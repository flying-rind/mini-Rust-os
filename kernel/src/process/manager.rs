use super::*;
use crate::*;
use alloc::collections::VecDeque;

#[derive(Default)]
pub struct TaskManager {
    runnable: VecDeque<Box<Task>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            runnable: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, t: Box<Task>) {
        self.runnable.push_back(t);
    }

    pub fn dequeue(&mut self) -> &'static mut Task {
        Box::leak(self.runnable.pop_front().unwrap())
    }

    pub fn resched(&mut self) {
        let cur = current();
        if cur.status == TaskStatus::Runnable {
            self.enqueue(unsafe { Box::from_raw(cur) });
        }
        let nxt = self.dequeue();
        if cur as *const _ != nxt as *const _ {
            cur.switch_to(nxt);
        }
    }
}
