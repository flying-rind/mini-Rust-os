//! 裸机的硬件接口实现
use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        #[path = "arch/x86_64/mod.rs"]
        pub mod arch;
    } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        #[path = "arch/riscv/mod.rs"]
        pub mod arch;
    } else if #[cfg(target_arch = "aarch64")] {
        #[path = "arch/aarch64/mod.rs"]
        pub mod arch;
    }
}

pub mod boot;
pub mod mem;

pub use arch::config::KernelConfig;
