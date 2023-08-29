#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

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
fn test_double_fault() {
    unsafe {
        *(0xdeadbeef0000 as *mut u64) = 42;
    };
}
