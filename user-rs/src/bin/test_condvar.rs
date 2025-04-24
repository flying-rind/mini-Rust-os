#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

extern crate alloc;

use alloc::vec;
use user_lib::{
    condvar_create, condvar_signal, condvar_wait, mutex_create, mutex_lock, mutex_unlock,
    thread_exit,
};
use user_lib::{thread_create, thread_join};

static mut A: usize = 0;

const CONDVAR_ID: usize = 0;
const MUTEX_ID: usize = 0;

unsafe fn first() -> () {
    // sleep(10);
    println!("First work, Change A --> 1 and wakeup Second");
    mutex_lock(MUTEX_ID);
    A = 1;
    condvar_signal(CONDVAR_ID);
    mutex_unlock(MUTEX_ID);
    thread_exit()
}

unsafe fn second() -> () {
    println!("Second want to continue,but need to wait A=1");
    mutex_lock(MUTEX_ID);
    while A == 0 {
        println!("Second: A is {}", A);
        condvar_wait(CONDVAR_ID, MUTEX_ID);
    }
    mutex_unlock(MUTEX_ID);
    println!("A is {}, Second can work now", A);
    thread_exit()
}

#[no_mangle]
pub fn main() -> i32 {
    // create condvar & mutex
    assert_eq!(condvar_create() as usize, CONDVAR_ID);
    assert_eq!(mutex_create() as usize, MUTEX_ID);
    // create threads
    let threads = vec![
        thread_create(first as usize, 0, 0),
        thread_create(second as usize, 0, 0),
    ];
    // wait for all threads to complete
    for thread in threads.iter() {
        thread_join(thread.unwrap());
    }
    println!("test_condvar passed!");
    0
}
