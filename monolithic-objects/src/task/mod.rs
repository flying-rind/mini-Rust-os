//! 宏内核任务类内核对象
extern crate alloc;

pub use alloc::sync::Arc;
pub use process::*;
pub use spin::Mutex;
pub use thread::*;

mod abi;
mod process;
mod thread;

/// 进程初始化
pub fn init() {
    unimplemented!()
}
