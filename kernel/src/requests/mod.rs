//! 用户向内核发送的请求模块
//!
//! 定义请求类型和内核线程处理请求的处理器
mod blk_processor;
mod fs_processor;
mod processor;

pub use fs_processor::*;
pub use processor::*;

use alloc::vec::Vec;

/// 用户向内核线程发送的请求
///
/// 用户发送请求时将其转化为字节，用户态再重新解析
pub type Request = Vec<u8>;
