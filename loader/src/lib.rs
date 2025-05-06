//! 微内核、宏内核、混合内核的加载器
#![no_std]
extern crate alloc;
extern crate log;
#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(feature = "hybrid")] {
        pub mod hybrid;
    }
}
