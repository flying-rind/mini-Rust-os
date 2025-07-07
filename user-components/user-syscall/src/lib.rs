//! 封装内核提供的系统调用给用户程序使用
#![no_std]

#[cfg(feature = "hybrid")]
pub mod hybrid;

#[cfg(feature = "monolithic")]
pub mod monolithic;

pub mod num;
extern crate alloc;
pub use alloc::string::String;

#[cfg(feature = "hybrid")]
pub use hybrid::*;
#[cfg(feature = "monolithic")]
pub use monolithic::*;

pub type SysResult = Result<usize, SysError>;
