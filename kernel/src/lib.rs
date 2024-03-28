#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(new_uninit)]
#![feature(panic_info_message)]

//! 包括内核主要模块和初始化部分，使集成测试程序和主程序可以复用大部分代码

extern crate alloc;
use core::{
    cell::UnsafeCell,
    mem,
    ops::{Deref, DerefMut},
    panic::PanicInfo,
};

pub use alloc::{boxed::Box, string::String, vec, vec::Vec};
pub use mem::{size_of, size_of_val, transmute};

#[macro_use]
// 驱动管理
// pub mod driver;
pub mod console;

pub mod pic;

// 内存管理
pub mod mm;

// 添加系统调用
pub mod syscall;

// 进程管理
pub mod process;

pub mod loader;

// 中断时切换栈和中断处理模块
pub mod trap;

// 简化版的x86_64库
pub mod my_x86_64;

/// 各类初始化函数
pub fn init(boot_info: &'static mut bootloader_api::BootInfo) {
    console::init();
    // 初始化中断和陷入
    trap::init();
    pic::init();
    // 初始化内存管理
    mm::init(&mut boot_info.memory_regions);
}

#[inline(always)]
pub const fn zero<T>() -> T {
    unsafe { mem::MaybeUninit::zeroed().assume_init() }
}

#[derive(Debug)]
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

#[no_mangle]
fn rust_oom() -> ! {
    panic!("rust_oom");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(l) = info.location() {
        println!(
            "[kernel] Panicked at {}:{} {}",
            l.file(),
            l.line(),
            info.message().unwrap()
        );
    } else {
        println!("[kernel] Panicked: {}", info.message().unwrap());
    }
    loop {}
}
