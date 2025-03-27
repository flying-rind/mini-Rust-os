//! 硬件平台的boot和初始化实现
use crate::hal_fn::boot::{__HalImpl, __HalTrait};
use crate::{KernelConfig, KernelHandler};

impl __HalTrait for __HalImpl {
    fn cmdline() -> String {}
    fn init_ram_disk() -> Option<&'static mut [u8]> {}
    fn primary_init() {}
    fn primary_init_early(cfg: KernelConfig, handler: &'static impl KernelHandler) {}
    fn secondary_init() {}
}
