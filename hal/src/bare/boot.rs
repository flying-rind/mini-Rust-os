//! 各硬件平台通用的的boot和初始化实现
use crate::hal_fn::boot::{__HalImpl, __HalTrait};
use crate::{KernelConfig, KernelHandler};
use alloc::string::String;

impl __HalTrait for __HalImpl {
    fn cmdline() -> String {
        unimplemented!()
    }
    fn init_ram_disk() -> Option<&'static mut [u8]> {
        unimplemented!()
    }
    fn primary_init() {
        unimplemented!()
    }
    fn primary_init_early(cfg: KernelConfig, handler: &'static impl KernelHandler) {
        unimplemented!()
    }
    fn secondary_init() {
        unimplemented!()
    }
}
