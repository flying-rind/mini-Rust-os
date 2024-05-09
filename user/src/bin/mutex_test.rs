#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use user_lib::{exit, thread_create, waittid};
use user_lib::{mutex_create, mutex_lock, mutex_unlock};

static mut A: usize = 0;
const PER_THREAD: usize = 1000;
const THREAD_COUNT: usize = 16;

unsafe fn f() -> ! {
    let mut t = 2usize;
    for _ in 0..PER_THREAD {
        mutex_lock(0);
        let a = &mut A as *mut usize;
        let cur = a.read_volatile();
        for _ in 0..500 {
            t = t * t % 10007;
        }
        a.write_volatile(cur + 1);
        mutex_unlock(0);
    }
    exit(t as i32)
}

#[no_mangle]
pub fn main() -> i32 {
    assert_eq!(mutex_create(), 0);
    let mut v = Vec::new();
    for _ in 0..THREAD_COUNT {
        v.push(thread_create(f as usize, 0) as usize);
    }
    for tid in v.iter() {
        waittid(*tid);
    }
    assert_eq!(unsafe { A }, PER_THREAD * THREAD_COUNT);
    0
}
