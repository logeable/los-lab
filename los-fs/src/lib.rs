#![no_std]

extern crate alloc;

pub mod cache;
pub mod device;
pub mod error;

pub const BLOCK_SIZE: usize = 512;
pub const BLOCK_CACHE_COUNT: usize = 16;
