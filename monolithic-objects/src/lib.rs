//! 宏内核对象
#![no_std]
pub use fs::ROOT_INODE;
pub use log::debug;
pub use signal::*;
pub use task::*;

extern crate alloc;

pub mod fs;
mod signal;
pub mod sync;
pub mod task;
