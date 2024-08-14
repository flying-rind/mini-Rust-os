#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(new_uninit)]
#![feature(panic_info_message)]
//! 包括内核主要模块和初始化部分，使集成测试程序和主程序可以复用大部分代码

#[macro_use]
extern crate log;
extern crate alloc;
use core::cell::UnsafeCell;
use core::{
    mem,
    ops::{Deref, DerefMut},
    panic::PanicInfo,
};

pub use easy_fs::BlockDevice;

pub use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
    rc::Rc,
    string::String,
    vec,
    vec::Vec,
};
pub use mem::{size_of, size_of_val, transmute};
use task::{KthreadType, CURRENT_KTHREAD};
pub use utils::*;

use crate::task::*;

// pub mod component;
pub mod drivers;
pub mod fs;
pub mod future;
pub mod kthread;
pub mod mm;
pub mod requests;
pub mod sync;
pub mod syscall;
pub mod task;
pub mod trap;
pub mod utils;

#[inline(always)]
pub const fn zero<T>() -> T {
    unsafe { mem::MaybeUninit::zeroed().assume_init() }
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
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }

    #[inline(always)]
    pub fn get(&self) -> &T {
        unsafe { &*self.0.get() }
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
        self.get_mut()
    }
}

#[no_mangle]
fn rust_oom() -> ! {
    panic!("rust_oom");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // 打印错误信息
    if let Some(l) = info.location() {
        println!(
            "\x1b[31m[kernel] Panicked at {}:{} {}\x1b[0m",
            l.file(),
            l.line(),
            info.message().unwrap()
        );
    } else {
        println!("[Kernel] Panicked: {}", info.message().unwrap());
    }
    // 若是内核服务线程崩溃了，尝试恢复错误
    let current_kthread = CURRENT_KTHREAD.get().as_ref().unwrap().clone();
    match current_kthread.ktype() {
        KthreadType::ROOT | KthreadType::EXECUTOR | KthreadType::UNKNOWN => {
            println!("[Panic handler] Cannot reboot!");
        }
        KthreadType::BLK | KthreadType::FS => {
            let current_req_id = current_kthread.current_request_id();
            println!(
                "\x1b[31m[Panic handler] Trying to Reboot..., the dangerous request(ID: {}) will be dropped!\x1b[0m",
                current_req_id
            );
            // 重启内核线程
            current_kthread.reboot(current_kthread.clone());
        }
    }
    loop {}
}
