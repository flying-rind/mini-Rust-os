#![no_std]
#![no_main]
#![feature(const_maybe_uninit_zeroed)]
// #![warn(missing_docs)]
// #![deny(warnings)]

//! 内核主函数
use bootloader_api::{config::Mapping, BootInfo, BootloaderConfig};
use core::cell::UnsafeCell;
use core::mem;
use core::ops::Deref;
use core::ops::DerefMut;
use kernel::{app::loader::list_apps, mem::KERNEL_OFFSET};

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

/// bootloader配置
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::FixedAddress(KERNEL_OFFSET as _));
    config
};

// 使用bootloader_api库提供的宏声明内核入口
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

/// 内核入口函数，参数为bootloader收集的硬件信息
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(boot_info);
    // kernel::mem::set_pagetable_to_user();
    // kernel::app::batch::init();
    // kernel::app::batch::run_next_app();
    // kernel::process::init();
    list_apps();
    kernel::process::init();
    unreachable!("test only");
}
