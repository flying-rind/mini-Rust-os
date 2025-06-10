//! 封装内核提供的系统调用给用户程序使用
#![no_std]

#[cfg(feature = "hybrid")]
pub mod hybrid;

#[cfg(feature = "monolithic")]
pub mod monolithic;
