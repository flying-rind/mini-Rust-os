//! 各硬件平台通用的抽象数据结构
pub(super) mod defs;
pub(super) mod mem;


pub mod console;

pub use defs::*;
pub use mem::*;
