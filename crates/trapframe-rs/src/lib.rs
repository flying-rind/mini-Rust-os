#![no_std]
#![feature(linkage)]
#![deny(warnings)]
#![cfg_attr(target_arch = "mips", feature(asm_experimental_arch))]

extern crate alloc;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
mod arch;

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
#[path = "arch/riscv/mod.rs"]
mod arch;

#[cfg(target_arch = "mips")]
#[path = "arch/mipsel/mod.rs"]
pub mod arch;

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
pub mod arch;

pub use arch::*;
