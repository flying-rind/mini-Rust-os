//! 硬件抽象层
pub use common::*;

mod common;
mod hal_fn;

// 硬件接口的具体实现
#[path = "bare/mod.rs"]
mod imp;
