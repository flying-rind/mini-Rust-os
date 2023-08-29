#![no_std]
#![no_main]
// // #![warn(missing_docs)]
// // #![deny(warnings)]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(const_maybe_uninit_zeroed)]
#![feature(new_uninit)]

//! 包括内核主要模块和初始化部分，使集成测试程序和主程序可以复用大部分代码

extern crate alloc;
use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

#[macro_use]
// 驱动管理
pub mod driver;

// 内存管理
pub mod mem;

// 任务管理
pub mod task;

// shell管理
pub mod shell;

// 添加系统调用
pub mod syscall;

// 进程管理
pub mod process;

// 测试管理
pub mod test;

// 中断时切换栈和中断处理模块
pub mod trap;

// 简化版的x86_64库
pub mod my_x86_64;

// 用户程序管理
pub mod app;

/// 各类初始化函数
pub fn init(boot_info: &'static mut bootloader_api::BootInfo) {
    // 初始化串口
    driver::serial::init();

    // 初始化FrameBuffer
    driver::fb::init(boot_info.framebuffer.as_mut().unwrap());

    // 初始化鼠标设备
    driver::mouse::init();

    // 初始化中断和陷入
    trap::init();

    // 初始化中断描述符表
    driver::idt::init();

    // 初始化中断控制器
    driver::pic::init();

    // 初始化内存管理
    mem::init(&mut boot_info.memory_regions);
}

/// 进入休眠状态
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[inline(always)]
pub const fn zero<T>() -> T {
    unsafe { MaybeUninit::zeroed().assume_init() }
}

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Cell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for Cell<T> {}

impl<T> Cell<T> {
    /// User is responsible to guarantee that inner struct is only used in
    /// uniprocessor.
    #[inline(always)]
    pub const fn new(val: T) -> Self {
        Self(UnsafeCell::new(val))
    }

    #[inline(always)]
    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

impl<T> Deref for Cell<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for Cell<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get()
    }
}

#[panic_handler]
// Rust语言要求的panic处理函数，通常由标准库提供，我们需要自己实现
fn panic(info: &core::panic::PanicInfo) -> ! {
    // 格式化打印PanicInfo，通过串口输出至终端
    serial_println!("{}", info);
    test::exit_qemu(test::QemuExitCode::Failed);
    hlt_loop();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("alloction error: {:?}", layout)
}

#[cfg(test)]
/// bootloader配置
pub static BOOTLOADER_CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::FixedAddress(
        mem::KERNEL_PHY_OFFSET as _,
    ));
    config
};

#[cfg(test)]
// 测试lib.rs需要单独定义入口
// 使用bootloader_api库提供的宏声明内核入口
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

#[cfg(test)]
/// 内核入口函数，参数为bootloader收集的硬件信息
fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    init(boot_info);
    test_main();
    hlt_loop();
}

#[test_case]
/// 测试用例示例
fn test_dummy() {
    assert_eq!(1, 1);
}
