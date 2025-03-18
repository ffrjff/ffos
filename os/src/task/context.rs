//! Implementation of [`TaskContext`]
use crate::trap::trap_ret_to_user_mod;

/// Task Context
#[derive(Copy, Clone, Debug)]
#[repr(C)]
/// TaskContext
pub struct TaskContext {
    /// return address after trap ret
    pub return_address: usize,
    /// sp
    sp: usize,
    /// callee_saved_register: s0~s11
    callee_saved_register: [usize; 12],
}

impl TaskContext {
    /// create a new task context
    pub fn new() -> Self {
        Self {
            return_address: 0,
            sp: 0,
            callee_saved_register: [0; 12],
        }
    }

    /// init task context return addr to trap return to start a ready task
    pub fn init(kernel_stack_sp: usize) -> Self {
        Self {
            return_address: trap_ret_to_user_mod as usize,
            sp: kernel_stack_sp,
            callee_saved_register: [0; 12],
        }
    }
}
