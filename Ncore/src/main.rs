#![no_std]
#![no_main]

//! 内核主函数
extern crate alloc;
use crate::alloc::string::ToString;
use alloc::string::String;
use alloc::vec;
use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping};
use loader::hybrid::kernel_main;

mod lang;

/// bootloader config
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    // 注意扩大内核栈大小，Bootloader默认为80KB
    config.kernel_stack_size = 200 * 1024 * 1024;
    config.mappings.physical_memory = Some(Mapping::FixedAddress(PHYS_OFFSET as _));
    config.mappings.kernel_stack = Mapping::FixedAddress(KERNEL_STACK_BASE as _);
    config
};

// 使用bootloader_api库提供的宏声明内核入口
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);
