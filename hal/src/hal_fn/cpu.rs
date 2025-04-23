//! CPU information

/// CPU信息
pub(crate) trait __HalTrait {
    /// Current CPU ID.
    fn cpu_id() -> u8 {
        0
    }

    /// Current CPU frequency in MHz.
    fn cpu_frequency() -> u16 {
        3000
    }

    /// Shutdown/reboot the machine.
    fn reset() -> ! {
        unimplemented!("cpu::rest")
    }
}

/// A struct that implements the hal interface.
pub(crate) struct __HalImpl;
/// Current CPU ID.
pub fn cpu_id() -> u8 {
    __HalImpl::cpu_id()
}

/// Current CPU frequency in MHz.
pub fn cpu_frequency() -> u16 {
    __HalImpl::cpu_frequency()
}

/// Shutdown/reboot the machine.
pub fn reset() -> ! {
    __HalImpl::reset()
}
