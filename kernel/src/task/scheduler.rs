//! 线程总调度器
use super::*;
use alloc::sync::Arc;
use trap::*;

/// 全局线程调度器
///
/// 统一管理内核线程和用户线程
pub struct Scheduler {}

impl Scheduler {
    /// 获取第一个需要运行的内核线程，
    ///
    /// 将其弹出全局队列，应当确保当前内核线程不在当前全局队列中
    pub fn get_first_kthread() -> Option<Arc<Kthread>> {
        let kthread_deque = KTHREAD_DEQUE.get_mut();
        for _ in 0..kthread_deque.len() {
            let kthread = kthread_deque.pop_front().unwrap();
            if kthread.need_schedule() {
                return Some(kthread.clone());
            } else {
                kthread_deque.push_back(kthread);
            }
        }
        None
    }

    /// 获取第一个需要运行的用户线程
    ///
    /// 若没有则返回None
    pub fn get_first_uthread() -> Option<Arc<Thread>> {
        let thread_deque = THREAD_DEQUE.get_mut();
        for _ in 0..thread_deque.len() {
            let thread = thread_deque.pop_front().unwrap();
            if thread.state() == ThreadState::Runnable {
                return Some(thread.clone());
            } else {
                thread_deque.push_back(thread);
            }
        }
        None
    }

    /// 当前内核线程放弃CPU，调度下一个就绪内核线程
    pub fn yield_current_kthread() {
        let current_kthread = CURRENT_KTHREAD.get().as_ref().unwrap().clone();
        let kthread = Scheduler::get_first_kthread();
        if let Some(kthread) = kthread {
            // [Debug]
            // println!(
            //     "[Debugger] `{}` switch to `{}`",
            //     current_kthread.name(),
            //     kthread.name()
            // );
            KTHREAD_DEQUE.get_mut().push_back(current_kthread.clone());
            // 修改全局变量
            *CURRENT_KTHREAD.get_mut() = Some(kthread.clone());
            current_kthread.switch_to(kthread);
        }
    }
}

/// 调度用户线程和内核线程
pub fn main_loop() {
    println!("[Kernel] Starting main loop...");
    loop {
        // 优先运行内核线程
        let kthread = Scheduler::get_first_kthread();
        if kthread.is_some() {
            // [Debug]
            // println!("`Root` switch to `{}`", kthread.as_ref().unwrap().name());
            // 将CPU交给服务线程或执行器
            let kthread = kthread.unwrap();
            let current_kthread = CURRENT_KTHREAD.get().as_ref().unwrap().clone();
            // 修改当前内核线程
            *CURRENT_KTHREAD.get_mut() = Some(kthread.clone());
            // 主线程入队
            KTHREAD_DEQUE.get_mut().push_back(current_kthread.clone());
            current_kthread.switch_to(kthread);
        } else {
            let uthread = Scheduler::get_first_uthread();
            // 运行用户线程
            if uthread.is_some() {
                let uthread = uthread.unwrap();
                // 修改当前线程
                *CURRENT_THREAD.get_mut() = Some(uthread.clone());
                // 持续运行用户线程直到其被挂起
                // [Debug]
                // println!("uthread running, pid {}", uthread.proc().unwrap().pid());
                while uthread.state() == ThreadState::Runnable {
                    uthread.run_until_trap();
                    handle_user_trap(uthread.clone(), &uthread.user_context());
                }
                // 此时线程已被挂起
                clear_current_thread();
            }
        }
    }
}

/// 清理当前线程
pub fn clear_current_thread() {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    // 根据线程状态进行清理
    match current_thread.state() {
        ThreadState::Suspended => {
            current_thread.set_state(ThreadState::Runnable);
            THREAD_DEQUE.get_mut().push_back(current_thread.clone());
        }
        ThreadState::Runnable | ThreadState::Waiting | ThreadState::Stop => {
            THREAD_DEQUE.get_mut().push_back(current_thread.clone());
        }
        ThreadState::Exited => {
            // 已退出时清理当前线程全局变量以drop线程
            current_thread.exit();
            *CURRENT_THREAD.get_mut() = None;
        }
    }
}
