#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

#[macro_use]
extern crate user_lib;

extern crate alloc;

use alloc::vec::Vec;
use user_lib::thread_exit;
use user_lib::{sem_create, sem_down, sem_up};
use user_lib::{thread_create, thread_join};

const SEM_MUTEX: usize = 0;
const SEM_EMPTY: usize = 1;
const SEM_EXISTED: usize = 2;
const BUFFER_SIZE: usize = 400;
static mut BUFFER: [usize; BUFFER_SIZE] = [0; BUFFER_SIZE];
static mut FRONT: usize = 0;
static mut TAIL: usize = 0;
const PRODUCER_COUNT: usize = 4;
const NUMBER_PER_PRODUCER: usize = 100;

unsafe fn producer(id: *const usize) -> () {
    let id = *id;
    for _ in 0..NUMBER_PER_PRODUCER {
        sem_down(SEM_EMPTY);
        sem_down(SEM_MUTEX);
        BUFFER[FRONT] = id;
        FRONT = (FRONT + 1) % BUFFER_SIZE;
        sem_up(SEM_MUTEX);
        sem_up(SEM_EXISTED);
    }
    thread_exit()
}

unsafe fn consumer() -> () {
    for _ in 0..PRODUCER_COUNT * NUMBER_PER_PRODUCER {
        sem_down(SEM_EXISTED);
        sem_down(SEM_MUTEX);
        print!("{} ", BUFFER[TAIL]);
        TAIL = (TAIL + 1) % BUFFER_SIZE;
        sem_up(SEM_MUTEX);
        sem_up(SEM_EMPTY);
    }
    println!("");
    thread_exit()
}

#[no_mangle]
pub fn main() -> i32 {
    // create semaphores
    assert_eq!(sem_create(1) as usize, SEM_MUTEX);
    assert_eq!(sem_create(BUFFER_SIZE) as usize, SEM_EMPTY);
    assert_eq!(sem_create(0) as usize, SEM_EXISTED);
    // create threads
    let ids: Vec<_> = (0..PRODUCER_COUNT).collect();
    let mut threads = Vec::new();
    for i in 0..PRODUCER_COUNT {
        threads.push(thread_create(
            producer as usize,
            &ids.as_slice()[i] as *const _ as usize,
            0,
        ));
    }
    threads.push(thread_create(consumer as usize, 0, 0));
    // wait for all threads to complete
    for thread in threads.iter() {
        thread_join(thread.unwrap());
    }
    println!("mpsc_sem passed!");
    0
}
