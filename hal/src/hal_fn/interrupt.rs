//! Interrupt management.
use crate::*;
pub(crate) trait __HalTriat {
    /// Suspend the CPU (also enable interrupts) and wait for an interrupt
    /// to occurs, then disable interrupts.
    fn wait_for_interrupt() {
        core::hint::spin_loop();
    }

    /// Is a valid IRQ number.
    fn is_valid_irq(vector: usize) -> bool {
        unimplemented!("interrupt::is_valid_irq");
    }

    /// Enable interrupts.
    fn intr_on() {
        unimplemented!("interrupt::intr_on");
    }

    /// Disable interrupts.
    fn intr_off() {
        unimplemented!("interrupt::intr_off");
    }

    /// Test weather interrupt is enabled.
    fn intr_get() -> bool {
        unimplemented!("interrupt::intr_get");
    }

    /// Disable IRQ.
    fn mask_irq(vector: usize) -> HalResult {
        unimplemented!("interrupt::mask_irq");
    }

    /// Enable IRQ.
    fn unmask_irq(vector: usize) -> HalResult {
        unimplemented!("interrupt::unmask_irq");
    }

    // /// ??? Configure the specified interrupt vector. If it is invoked, it must be
    // /// invoked prior to interrupt registration.
    // fn configure_irq(vector: usize, tm: Irq)

    // /// Add an interrupt handler to an IRQ.
    // fn register_irq_handler(vector: usize, handler: IrqHandler) -> HalResult {
    //     unimplemented!("interrupt::register_irq_handler");
    // }

    /// Remove the interrupt handler to an IRQ.
    fn unregister_irq_handler(vector: usize) -> HalResult {
        unimplemented!("interrupt::unregister_irq_handler");
    }

    /// Handle IRQ.
    fn handle_irq(vector: usize) {
        unimplemented!("interrupt::handle_irq");
    }

    // More
}

pub(crate) struct __HalImpl;

/// Suspend the CPU (also enable interrupts) and wait for an interrupt
/// to occurs, then disable interrupts.
pub fn wait_for_interrupt() {
    __HalImpl::wait_for_interrupt();
}

/// Is a valid IRQ number.
pub fn is_valid_irq(vector: usize) -> bool {
    __HalImpl::is_valid_irq(vector)
}

/// Enable interrupts.
pub fn intr_on() {
    __HalImpl::intr_on();
}

/// Disable interrupts.
pub fn intr_off() {
    __HalImpl::intr_off();
}

/// Test weather interrupt is enabled.
pub fn intr_get() -> bool {
    __HalImpl::intr_get()
}

/// Disable IRQ.
pub fn mask_irq(vector: usize) -> HalResult {
    __HalImpl::mask_irq(vector)
}

/// Enable IRQ.
pub fn unmask_irq(vector: usize) -> HalResult {
    __HalImpl::unmask_irq(vector)
}

// /// ??? Configure the specified interrupt vector. If it is invoked, it must be
// /// invoked prior to interrupt registration.
// fn configure_irq(vector: usize, tm: Irq)

// /// Add an interrupt handler to an IRQ.
// fn register_irq_handler(vector: usize, handler: IrqHandler) -> HalResult {
//     unimplemented!("interrupt::register_irq_handler");
// }

/// Remove the interrupt handler to an IRQ.
pub fn unregister_irq_handler(vector: usize) -> HalResult {
    __HalImpl::unregister_irq_handler(vector)
}

/// Handle IRQ.
pub fn handle_irq(vector: usize) {
    __HalImpl::handle_irq(vector);
}

// More
