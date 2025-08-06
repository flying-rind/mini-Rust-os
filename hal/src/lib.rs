//! 硬件抽象层
#![no_std]
#[macro_use]
extern crate log;
extern crate alloc;
pub use common::*;

// 通用模块
mod common;
// 硬件抽象层提供给上层的接口定义
pub mod hal_fn;
// 硬件接口的具体实现
#[path = "bare/mod.rs"]
mod imp;
// 内核处理函数
mod kernel_handler;
// 内核配置初始化
mod config;
// 工具库
mod utils;

pub use imp::*;
pub use kernel_handler::KernelHandler;
