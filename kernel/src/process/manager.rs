use core::mem::transmute;

use alloc::collections::VecDeque;

use super::{
    current,
    task::{Task, TaskPtr, TaskStatus},
};

#[derive(Default)]
pub struct TaskManager {
    runnable: VecDeque<TaskPtr>,
}

impl TaskManager {
    pub fn enquene(&mut self, t: &mut Task) {
        self.runnable.push_back(unsafe { transmute(t) });
    }

    pub fn dequene(&mut self) -> TaskPtr {
        self.runnable.pop_front().unwrap()
    }

    pub fn clear_zombie(&mut self) {
        let len: usize = self.runnable.len();
        for _ in 0..len {
            let t = self.runnable.pop_front().unwrap();
            if t.status == TaskStatus::Runnable {
                self.runnable.push_back(t);
            }
        }
    }

    pub fn resched(&mut self) {
        serial_println!("[TaskManager] reschedule happened.");
        let cur = current();
        if cur.status == TaskStatus::Runnable {
            self.enquene(cur);
        }
        let nxt = self.dequene();
        if cur as *const _ != nxt as *const _ {
            cur.switch_to(nxt);
        }
    }
}
