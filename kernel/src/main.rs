#![no_std]
#![no_main]

//! 内核主函数
extern crate alloc;
use crate::alloc::string::ToString;
use alloc::string::String;
use alloc::vec;
use bootloader_api::{config::Mapping, BootInfo, BootloaderConfig};
use kernel::{
    kthread,
    mm::{KERNEL_STACK_BASE, PHYS_OFFSET},
    task::Kthread,
    trap::{main_loop, Process},
};

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

/// 内核入口函数，参数为bootloader收集的硬件信息
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // 初始化串口
    kernel::serial::init(0x3f8);
    // 初始化堆
    kernel::mm::heap_init();
    // 初始化中断描述符表
    kernel::trap::init();
    // 初始化内存管理
    kernel::mm::init(&mut boot_info.memory_regions);
    // 初始化中断
    kernel::pic::init();
    // 初始化驱动
    kernel::drivers::init();
    // 初始化文件系统
    kernel::fs::init();
    // 创建根内核线程
    Kthread::new_root();
    // 初始化内核服务线程
    kthread::init();
    // 创建并启动shell进程
    let test_args = vec![
        "testarg1".to_string(),
        "testarg2".to_string(),
        "testarg3".to_string(),
    ];
    let shell_str = "shell";
    let shell_process = Process::new(String::from(shell_str), &shell_str, Some(test_args)).unwrap();
    shell_process.root_thread().resume();

    // 跳转到用户态
    main_loop();
    unreachable!("Should never reach here");
}
