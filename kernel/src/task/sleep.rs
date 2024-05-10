//! 实现进程阻塞睡眠
use super::*;
use crate::*;
use alloc::collections::BinaryHeap;

/// 睡眠中的线程
struct SleepingThread {
    // 线程超时的时间
    expire_ms: usize,
    thread: ThreadPtr,
}

// 为SleepingThread实现几个比较的trait
// 使其能放入优先队列中
impl PartialEq for SleepingThread {
    fn eq(&self, other: &Self) -> bool {
        self.expire_ms == other.expire_ms
    }
}

impl Eq for SleepingThread {}

impl PartialOrd for SleepingThread {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SleepingThread {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // BinaryHeap是大根堆，故这里取反
        self.expire_ms.cmp(&other.expire_ms).reverse()
    }
}

/// 睡眠中线程的优先队列
static TIMERS: Cell<BinaryHeap<SleepingThread>> =
    unsafe { transmute(Vec::<SleepingThread>::new()) };

/// 当前线程睡眠，加入睡眠优先队列
pub fn add_timer(ms: usize) {
    TIMERS.get().push(SleepingThread {
        expire_ms: *pic::TICKS,
        thread: current(),
    })
}

/// 从睡眠线程队列中移除已退出线程
///
/// 在线程exit后调用
pub fn clear_zombie_timer() {
    let timers = core::mem::replace(TIMERS.get(), BinaryHeap::new());
    for sleep_t in timers {
        if sleep_t.thread.state == ThreadState::Runnable {
            TIMERS.get().push(sleep_t);
        }
    }
}

/// 检查是否有线程可唤醒
///
/// 若有将其唤醒并从队列中移除
///
/// 在每个时钟中断时调用
pub fn check_timer() {
    let current_ms = *pic::TICKS;
    while let Some(t) = TIMERS.get().peek() {
        // 唤醒最早线程
        if t.expire_ms <= current_ms {
            sched_unblock(unsafe { &mut *(t.thread as *const _ as *mut _) });
            TIMERS.get().pop();
        } else {
            break;
        }
    }
}
