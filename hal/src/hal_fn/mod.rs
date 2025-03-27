//! 硬件抽象层暴露给上层内核对象的接口,这里没有使用Zcore中的复杂宏的技巧，
//! 而是直接显式的给出了模块定义

use crate::{PhysAddr, VirtAddr};
use core::ops::Range;

/// Bootstrap and initialization
pub mod boot;
/// Physical memory operations.
pub mod mem;
