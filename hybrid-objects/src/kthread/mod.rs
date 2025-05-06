//! 内核线程执行服务模块

use crate::requests::*;
use crate::task::Kthread;
use crate::task::KthreadType;
use crate::task::KTHREAD_MAP;

use alloc::string::ToString;

pub use executor::*;
pub use for_test::*;
pub use processor::*;

pub mod executor;
pub mod for_test;
pub mod processor;

/// 内核服务线程初始化，建立重要的内核服务线程
pub fn init() {
    // 创建内核协程执行器线程
    Kthread::new(
        "Executor".to_string(),
        executor_entry as _,
        None,
        KthreadType::EXECUTOR,
    );
    // 创建文件系统服务线程
    let fs_processor = FsProcessor::new();
    let fs_kthread = Kthread::new(
        "Fs-server".to_string(),
        processor_entry as _,
        Some(fs_processor),
        KthreadType::FS,
    );
    KTHREAD_MAP.get_mut().insert(KthreadType::FS, fs_kthread);
}
