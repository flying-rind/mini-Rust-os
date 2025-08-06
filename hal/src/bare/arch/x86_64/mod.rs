//! x86_64架构的硬件抽象接口实现
#![allow(unused)]
pub mod config;
pub mod cpu;
pub mod drivers;
pub mod interrupt;
pub mod mem;
pub mod vm;

/// x86_64初始化
pub fn primary_init() {
    drivers::drivers_init();
}
