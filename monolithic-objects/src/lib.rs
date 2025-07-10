//! 宏内核对象
#![no_std]
pub use hybrid_objects::fs::ROOT_INODE;
pub use log::debug;
pub use task::*;

extern crate alloc;

pub mod fs;
pub mod sync;
pub mod task;
