//! 常用定义
use bitflags::bitflags;
use numeric_enum_macro::numeric_enum;

/// The error type which is returned from HAL functions.
pub enum HalError {
    Common,
    // TODO: More
}
/// The result type returned by HAL functions.
pub type HalResult<T = ()> = core::result::Result<T, HalError>;

bitflags! {
    /// Generic memory flags.
    pub struct MMUFlags: usize {
        #[allow(clippy::identity_op)]
        const CACHE_1   = 1 << 0;
        const CACHE_2   = 1 << 1;
        const READ      = 1 << 2;
        const WRITE     = 1 << 3;
        const EXECUTE   = 1 << 4;
        const USER      = 1 << 5;
        const HUGE_PAGE = 1 << 6;
        const DEVICE    = 1 << 7;
        const RXW = Self::READ.bits | Self::WRITE.bits | Self::EXECUTE.bits;
    }
}

numeric_enum! {
    #[repr(u32)]
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    /// Generic cache policy.
    pub enum CachePolicy {
        Cached = 0,
        Uncached = 1,
        UncachedDevice = 2,
        WriteCombining = 3,
    }
}

/// The smallest size of a page (4K).
pub const PAGE_SIZE: usize = super::mem::PageSize::Size4K as usize;
