//! 内核线程执行服务模块

use crate::requests::*;
use crate::task::KTHREAD_MAP;
use crate::task::Kthread;
use crate::task::KthreadType;

use alloc::string::ToString;

pub use processor::*;

mod executor;
mod for_test;
mod processor;

/// 内核服务线程初始化，建立重要的内核服务线程
pub fn init() {
    // Create executor kthread.
    Kthread::new(
        "Executor".to_string(),
        executor::executor_entry as _,
        None,
        KthreadType::EXECUTOR,
    );
    // 创建根内核线程
    Kthread::new_root();
    // Create blk kthread.
    let blk_processor = BlkProcessor::new();
    let blk_kthread = Kthread::new(
        "Blk-Server".to_string(),
        processor_entry as _,
        Some(blk_processor),
        KthreadType::BLK,
    );
    KTHREAD_MAP.get_mut().insert(KthreadType::BLK, blk_kthread);

    // Create fs kthread.
    let fs_processor = FsProcessor::new();
    let fs_kthread = Kthread::new(
        "Fs-server".to_string(),
        processor_entry as _,
        Some(fs_processor),
        KthreadType::FS,
    );

    KTHREAD_MAP.get_mut().insert(KthreadType::FS, fs_kthread);
}
