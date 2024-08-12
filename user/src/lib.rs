//! 用户态的程序入口

#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
mod lang_items;

extern crate alloc;
extern crate bitflags;

use alloc::vec::Vec;
use buddy_system_allocator::LockedHeap;
pub use user_syscall::*;

const USER_HEAP_SIZE: usize = 0x40000;
static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap<32> = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[no_mangle]
#[link_section = ".text.entry"]
/// 用户态程序入口
///
/// 栈的情况：
///
/// ----high
///
/// \0
///
/// ptr2
///
/// ptr1    <--argv
///
/// str1
///
/// str2
///
/// ----low
pub extern "C" fn _start(argc: usize, argv: usize) -> ! {
    init_heap();
    // 从用户栈栈顶中读取argc和argv指针
    let mut v: Vec<&'static str> = Vec::new();
    for i in 0..argc {
        let str_start =
            unsafe { ((argv + i * core::mem::size_of::<usize>()) as *const usize).read_volatile() };
        let len = (0usize..)
            .find(|i| unsafe { ((str_start + *i) as *const u8).read_volatile() == 0 })
            .unwrap();
        v.push(
            core::str::from_utf8(unsafe {
                core::slice::from_raw_parts(str_start as *const u8, len)
            })
            .unwrap(),
        );
    }
    // 调用应用主函数
    let exit_code = main(argc, v.as_slice());
    proc_exit(exit_code as _);
    panic!("Should never reach after proc_exit");
}

/// 用户态初始化堆内存
pub fn init_heap() {
    // 初始化堆内存分配器
    unsafe {
        HEAP.lock()
            .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
}

#[linkage = "weak"]
#[no_mangle]
fn main(_argc: usize, _argv: &[&str]) -> isize {
    panic!("Cannot find main!");
}
