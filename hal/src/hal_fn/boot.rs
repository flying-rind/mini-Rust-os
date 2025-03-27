//! Bootstrap and initialization

use crate::imp::KernelConfig;
use crate::kernel_handler::KernelHandler;

/// 内核引导和初始化
pub(crate) trait __HalTrait {
    /// The kernel command line.
    ///
    /// TODO: use `&'a str` as return type.
    fn cmdline() -> String {
        unimplemented!("boot::comline");
    }

    /// Returns the slice of the initial RAM disk, or `None` if not exist.
    fn init_ram_disk() -> Option<&'static mut [u8]> {
        unimplemented!("boot::init_ram_disk");
    }

    /// Initialize the primary CPU at an early stage (before the physical frame allocator).
    fn primary_init_early(cfg: KernelConfig, handler: &'static impl KernelHandler) {
        unimplemented!("boot::primary_init_early");
    }

    /// The main part of the primary CPU initialization.
    fn primary_init() {
        unimplemented!("boot::primary");
    }

    /// Initialize the secondary CPUs.
    fn secondary_init() {
        unimplemented!("boot::secondary::init");
    }
}

/// A struct that implements the hal interface.
pub(crate) struct __HalImpl;

/// The kernel command line.
///
/// TODO: use `&'a str` as return type.
pub fn cmdline() -> String {
    __HalImpl::cmdline()
}

/// Returns the slice of the initial RAM disk, or `None` if not exist.
pub fn init_ram_disk() -> Option<&'static mut [u8]> {
    __HalImpl::init_ram_disk()
}

/// Initialize the primary CPU at an early stage (before the physical frame allocator).
pub fn primary_init_early(cfg: KernelConfig, handler: &'static impl KernelHandler) {
    __HalImpl::primary_init_early(cfg, handler)
}

/// The main part of the primary CPU initialization.
pub fn primary_init() {
    __HalImpl::primary_init();
}

/// Initialize the secondary CPUs.
pub fn secondary_init() {
    __HalImpl::secondary_init();
}
