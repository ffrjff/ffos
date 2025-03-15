//! Constants used in rCore

pub const USER_STACK_SIZE: usize = 4096;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;

pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;

pub const MAX_APP_NUM: usize = 4;

pub const APP_BASE_ADDRESS: usize = 0x80400000;

pub const APP_SIZE_LIMIT: usize = 0x20000;

pub const PAGE_SIZE_BITS: usize = 0xc;
pub const PAGE_SIZE: usize = 0x1000;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
/*
#[cfg(feature = "board_k210")]
pub const CLOCK_FREQ: usize = 403000000 / 62;

#[cfg(feature = "board_qemu")]
pub const CLOCK_FREQ: usize = 12500000;
*/
pub use crate::board::{CLOCK_FREQ, MEMORY_END, MMIO};
