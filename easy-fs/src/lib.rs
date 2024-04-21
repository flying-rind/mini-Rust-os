#![no_std]
#[macro_use]
extern crate lazy_static;
extern crate alloc;


mod block_cache;
mod block_dev;
mod bitmap;
mod layout;

pub const BLOCK_SIZE: usize = 512;