//! 中断描述符表模块

use crate::driver::pic::{notify_eoi, InterruptIndex};
use crate::driver::serial::receive;
use crate::process::current_yield;
use crate::zero;
use crate::Cell;
use spin::Lazy;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.double_fault.set_handler_fn(double_fault_handler);
    idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
    // idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);
    idt[InterruptIndex::Com1 as usize].set_handler_fn(com1_interrupt_handler);
    // idt[InterruptIndex::Mouse as usize].set_handler_fn(mouse_interrupt_handler);
    idt.general_protection_fault.set_handler_fn(gpfault_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt
});

static TICKS: Cell<usize> = zero();

/// 初始化中断描述符表
pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn gpfault_handler(stack_frame: InterruptStackFrame, code: u64) {
    panic!(
        "EXCEPTION: Genral Protection with code {}\n{:#?}",
        code, stack_frame
    );
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    code: PageFaultErrorCode,
) {
    panic!(
        "EXCEPTION: Page Fault with code {:#?}\n{:#?}",
        code, stack_frame
    );
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // crate::task::sleep::timer_tick();
    *TICKS.get() += 1;
    notify_eoi(InterruptIndex::Timer as u8);
    if *TICKS.get() % 5 == 0 {
        current_yield();
    }
}

// extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
//     use x86_64::instructions::port::Port;
//     let mut port = Port::new(0x60);
//     let scancode: u8 = unsafe { port.read() };
//     // crate::task::keyboard::add_scancode(scancode);
//     notify_eoi(InterruptIndex::Keyboard as u8);
// }

extern "x86-interrupt" fn com1_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let chr = receive();
    serial_print!("{}", chr as char);
    notify_eoi(InterruptIndex::Com1 as u8);
}
