//! x86的中断相关操作
use crate::hal_fn::interrupt::{__HalImpl, __HalTriat};

impl __HalTriat for __HalImpl {
    fn wait_for_interrupt() {
        unimplemented!()
    }

    fn is_valid_irq(vector: usize) -> bool {
        unimplemented!()
    }

    fn intr_on() {
        unimplemented!()
    }

    fn intr_off() {
        unimplemented!()
    }

    fn intr_get() -> bool {
        unimplemented!()
    }

    fn mask_irq(vector: usize) -> crate::HalResult {
        unimplemented!()
    }

    fn unmask_irq(vector: usize) -> crate::HalResult {
        unimplemented!()
    }

    fn unregister_irq_handler(vector: usize) -> crate::HalResult {
        unimplemented!()
    }

    fn handle_irq(vector: usize) {
        unimplemented!()
    }
}
