#![no_std]
#![no_main]
// #![warn(missing_docs)]
// #![deny(warnings)]

//! 内核主函数
use bootloader_api::{config::Mapping, BootInfo, BootloaderConfig};
use kernel::{app::loader::list_apps, mem::KERNEL_PHY_OFFSET};

/// bootloader配置
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::FixedAddress(KERNEL_PHY_OFFSET as _));
    config
};

// 使用bootloader_api库提供的宏声明内核入口
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

/// 内核入口函数，参数为bootloader收集的硬件信息
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(boot_info);
    kernel::fb_println!("Use the \"help\" command to get usage information.");
    kernel::fb_print!("[rust_os] >>> ");

    // kernel::mem::set_pagetable_to_user();

    // kernel::app::batch::init();
    // kernel::app::batch::run_next_app();
    // kernel::process::init();
    list_apps();
    kernel::process::init();
    unreachable!("test only");

    // 添加异步任务并执行
    // let mut executor = Executor::new();
    // executor.spawn(Task::new(kernel::task::keyboard::print_keypresses()));
    // executor.spawn(Task::new(kernel::task::mouse::print_mousemovements()));
    // executor.run();
    // unreachable!();
}
