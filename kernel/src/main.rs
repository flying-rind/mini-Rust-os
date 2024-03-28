#![no_std]
#![no_main]

//! 内核主函数
use bootloader_api::{config::Mapping, BootInfo, BootloaderConfig};
use kernel::{loader::list_apps, mm::KERNEL_STACK_ADDRESS, mm::PHYS_OFFSET};

/// bootloader config
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::FixedAddress(PHYS_OFFSET as _));
    config.mappings.kernel_stack = Mapping::FixedAddress(KERNEL_STACK_ADDRESS as _);
    config
};

// 使用bootloader_api库提供的宏声明内核入口
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

/// 内核入口函数，参数为bootloader收集的硬件信息
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(boot_info);
    list_apps();
    kernel::process::init();
    // unreachable!("test only");
}
