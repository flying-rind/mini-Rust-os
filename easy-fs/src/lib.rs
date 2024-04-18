#![no_std]

extern crate alloc;

mod block_cache;
mod block_dev;

pub const BLOCK_SIZE: usize = 512;