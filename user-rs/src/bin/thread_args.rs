#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::vec::Vec;
use user_syscall::{thread_create, thread_exit, thread_join};

struct Argument {
    pub ch: char,
    #[allow(unused)]
    pub rc: i32,
}

fn thread_print(arg: *const Argument) -> () {
    // println!("Thread created");
    let arg = unsafe { &*arg };
    for _ in 0..1000 {
        print!("{}", arg.ch);
    }
    thread_exit();
}

#[no_mangle]
pub fn main() -> i32 {
    let mut v = Vec::new();
    let args = [
        Argument { ch: 'a', rc: 1 },
        Argument { ch: 'b', rc: 2 },
        Argument { ch: 'c', rc: 3 },
    ];
    for arg in args.iter() {
        v.push(thread_create(
            thread_print as usize,
            arg as *const _ as usize,
            0,
        ));
    }
    for tid in v.iter() {
        thread_join(tid.unwrap());
        println!("Thread#{} exited", tid.unwrap());
    }
    println!("main thread exited. Thread with args test passed");
    0
}
