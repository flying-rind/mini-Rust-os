#![no_std]
#![no_main]

//! 内核主函数
extern crate alloc;
use bootloader_api::{BootloaderConfig, config::Mapping};
use hybrid_objects::mm::PHYS_OFFSET;
use hybrid_objects::mm::KERNEL_STACK_BASE;
use hybrid_objects::task::Kthread;
use bootloader_api::BootInfo;
use hybrid_objects::Process;
use alloc::string::String;
use alloc::vec;
use alloc::string::ToString;
use loader::hybrid::main_loop;
use log::warn;


mod lang;
mod logging;

/// bootloader config
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    // 注意扩大内核栈大小，Bootloader默认为80KB
    config.kernel_stack_size = 200 * 1024 * 1024;
    config.mappings.physical_memory = Some(Mapping::FixedAddress(PHYS_OFFSET as _));
    config.mappings.kernel_stack = Mapping::FixedAddress(KERNEL_STACK_BASE as _);
    config
};

// 内核入口函数，参数为bootloader收集的硬件信息
pub fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // 初始化串口
    hal::hal_fn::boot::primary_init();
    // 初始化日志
    logging::init();
    warn!("Test for warn");
    // 初始化堆
    hybrid_objects::mm::heap_init();
    // 初始化中断描述符表
    hybrid_objects::trap::init();
    // 初始化内存管理
    hybrid_objects::mm::init(&mut boot_info.memory_regions);
    // 初始化中断
    hybrid_objects::pic::init();
    // 初始化驱动
    hybrid_objects::drivers::init();
    // 初始化文件系统
    hybrid_objects::fs::init();
    // 创建根内核线程
    Kthread::new_root();
    // 初始化内核服务线程
    hybrid_objects::kthread::init();
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

// 使用bootloader_api库提供的宏声明内核入口
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);
