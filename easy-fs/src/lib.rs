#![no_std]
#[macro_use]
extern crate lazy_static;
extern crate alloc;

use bitmap::Bitmap;
use block_dev::BlockDevice;
use layout::*;
use block_cache::{get_block_cache, block_cache_sync_all};


mod block_cache;
mod block_dev;
mod bitmap;
mod layout;
mod efs;

pub const BLOCK_SIZE: usize = 512;