//! 用户请求处理器内核线程的统一入口
//! 内部通过内核线程的processor成员来具体服务

use crate::println;
use crate::task::KthreadState;
use crate::task::{Scheduler, CURRENT_KTHREAD};

/// 服务内核线程统一入口，内部通过内核线程的
/// processor对象来具体处理请求
pub fn processor_entry() {
    // 获取内核线程
    let kthread = CURRENT_KTHREAD.get().as_ref().unwrap().clone();
    // 获取请求处理器
    let processor = kthread.processor();
    assert!(processor.is_some());
    let processor = processor.unwrap();

    // 循环响应请求
    loop {
        // 获取请求
        let (req, req_id) = match kthread.get_first_request() {
            Some((req, req_id)) => {
                kthread.set_current_request_id(req_id);
                (req, req_id)
            }
            None => {
                // 请求队列为空，则设置自己为Idle，放弃CPU直到请求入队时改变状态为NeedRun
                kthread.set_state(KthreadState::Idle);
                Scheduler::yield_current_kthread();
                continue;
            }
        };
        // 处理请求
        processor.process_request(req);
        // 响应请求，唤醒等待协程
        kthread.wake_request(req_id);
        println!(
            "\x1b[34m[{}] Request {} processed over!\x1b[0m",
            kthread.name(),
            req_id,
        );
    }
}
