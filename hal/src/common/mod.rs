//! 各硬件平台通用的抽象数据结构
pub(super) mod defs;
pub(super) mod mem;

pub mod abi;
pub mod console;
pub mod cstr;
pub mod error;
pub mod time;
pub mod user;

pub use abi::*;
pub use cstr::*;
pub use defs::*;
pub use error::*;
pub use mem::*;
