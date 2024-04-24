#![no_std]
extern crate alloc;
extern crate lazy_static;

use bitmap::Bitmap;
use block_cache::{block_cache_sync_all, get_block_cache};
pub use block_dev::BlockDevice;
pub use efs::EasyFileSystem;
use layout::*;
pub use vfs::Inode;

mod bitmap;
mod block_cache;
mod block_dev;
mod efs;
mod layout;
mod vfs;

pub const BLOCK_SIZE: usize = 512;
