#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use kernel::serial_println;

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

/// bootloader配置
pub static BOOTLOADER_CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::FixedAddress(
        kernel::mem::KERNEL_PHY_OFFSET as _,
    ));
    config
};

// 使用bootloader_api库提供的宏声明内核入口
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

/// 内核入口函数，参数为bootloader收集的硬件信息
fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    kernel::init(boot_info);
    test_main();
    kernel::hlt_loop();
}

#[test_case]
fn simple_alloc() {
    let heap_value = Box::new(41);
    assert_eq!(*heap_value, 41);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }

    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..10_000 {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}
