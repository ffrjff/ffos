#![no_std]
extern crate alloc;
mod block_cache;
mod block_dev;

pub const BLOCK_SZ: usize = 512;
// use block_cache::{block_cache_sync_all, get_block_cache};
pub use block_dev::BlockDevice;

