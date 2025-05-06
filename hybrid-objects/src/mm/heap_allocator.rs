use crate::mm::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn heap_init() {
    #[allow(static_mut_refs)]
    let heap_start = unsafe{HEAP_SPACE.as_ptr() as usize};
    unsafe {
        HEAP_ALLOCATOR
        .lock()
        .init(heap_start, KERNEL_HEAP_SIZE);
    }
}
