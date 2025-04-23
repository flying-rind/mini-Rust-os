//! 硬件抽象层暴露给上层内核对象的接口,这里没有使用Zcore中的复杂宏的技巧，
//! 而是直接显式的给出了模块定义

use crate::{PhysAddr, VirtAddr};
use core::ops::Range;

/// Bootstrap and initialization
pub mod boot;
/// CPU information
pub mod cpu;
/// Interrupt management.
pub mod interrupt;
/// Physical memory operations.
pub mod mem;
/// Virtual memory operations.
pub mod vm;
