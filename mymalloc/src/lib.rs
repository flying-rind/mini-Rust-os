//! Runs in userspace.

#![no_std]
#![feature(alloc_error_handler)]
extern crate alloc;
use alloc::collections::BTreeMap;
use buddy_system_allocator::LockedHeap;
use core::alloc::Layout;
use user_syscall::println;

use lazy_static::lazy_static;
use spin::Mutex;

pub const USER_HEAP_BASE: usize = 0x0000_7F00_0000_0000;
const USER_HEAP_SIZE: usize = 0x8 * 1024 * 1024;

#[global_allocator]
static HEAP: LockedHeap<32> = LockedHeap::new();

#[unsafe(no_mangle)]
#[doc(hidden)]
pub extern "C" fn init_rust_runtime() {
    let start_addr = USER_HEAP_BASE;
    println!(
        "Init user heap!, heap start addr: {:#x}, end addr: {:#x}",
        start_addr,
        start_addr + USER_HEAP_SIZE
    );
    unsafe {
        HEAP.lock().init(start_addr, USER_HEAP_SIZE);
    }
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

lazy_static! {
    pub static ref MALLOC_LAYOUTS: Mutex<BTreeMap<usize, Layout>> = Mutex::new(BTreeMap::new());
}

/// musl-libc calls this to do malloc().
#[unsafe(no_mangle)]
pub extern "C" fn mymalloc(size: usize) -> isize {
    if size == 0 {
        return 0;
    }
    let lay = Layout::array::<u8>(size).expect("Failed to alloc");
    let ptr = unsafe { alloc::alloc::alloc(lay) };
    if ptr.is_null() {
        panic!("failed to malloc!");
    }
    let mut layouts = MALLOC_LAYOUTS.lock();
    layouts.insert(ptr as usize, lay);
    println!("alloc ptr: {:?}, size: {}", ptr, size);
    ptr as isize
}

/// musl-libc calls this to do malloc().
#[unsafe(no_mangle)]
pub extern "C" fn myrealloc(_size: usize) -> isize {
    // currently do nothing.
    panic!()
}

/// musl-libc calls this to do free().
#[unsafe(no_mangle)]
pub extern "C" fn myfree(ptr: usize) {
    let mut layouts = MALLOC_LAYOUTS.lock();
    match layouts.remove(&ptr) {
        Some(lay) => unsafe {
            alloc::alloc::dealloc(ptr as *mut u8, lay);
        },
        None => return,
    };
    println!("free, ptr: {:x?}", ptr);
}

#[panic_handler]
fn panic_handler(_panic_info: &core::panic::PanicInfo) -> ! {
    panic!("malloc panic!");
}
