//! 宏内核对象
#![no_std]
pub use log::debug;
pub use task::*;

extern crate alloc;

mod task;
