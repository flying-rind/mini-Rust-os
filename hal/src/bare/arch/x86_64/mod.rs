//! x86_64架构的硬件抽象接口实现
pub mod config;
pub mod cpu;
pub mod interrupt;
pub mod mem;
pub mod vm;
pub mod drivers;

/// x86_64初始化
pub fn primary_init() {
    drivers::drivers_init();
}