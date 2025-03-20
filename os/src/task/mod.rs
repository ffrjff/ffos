//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;
mod pid;
mod manager;
mod processor;


#[allow(unused)]
use crate::mm::frame_alloc;
#[allow(unused)]
use crate::mm::page_table::PTEFlags;
#[allow(unused)]
use crate::config::PAGE_SIZE;
#[allow(unused)]
use crate::mm::region::kernel_region::KernelRegion;
use crate::sbi::shutdown;
use crate::loader::get_app_data_by_name;
use alloc::sync::Arc;
use lazy_static::*;
use task::{ProcessControlBlock, ProcessStatus};
#[allow(unused)]
use log::info;
pub use context::TaskContext;
#[allow(unused)]
use riscv::register::satp;
#[allow(unused)]
use super::mm::address_space::KERNEL_SPACE;
#[allow(unused)]
use crate::mm::address::{VirtAddr, PhysAddr, VirtPageNum, PhysPageNum};
pub use pid::PidTracker;

pub use manager::add_process;
pub use pid::{pid_alloc, PidAllocator};
pub use processor::*;
pub use processor::{
    clone_current_process, get_current_trap_context, get_current_user_token, run_processes, schedule, take_current_process,
    Processor,
};

/// basic process: pid 0
pub const IDLE_PID: usize = 0;

lazy_static! {
    ///Globle process that init user shell
    pub static ref INITPROC: Arc<ProcessControlBlock> = Arc::new(ProcessControlBlock::new(
        get_app_data_by_name("initproc").unwrap(),
        0,
    ));
}


/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_process_and_run_next() {
    // There must be an application running.
    let process = take_current_process().unwrap();

    // ---- access current TCB exclusively
    let mut process_inner = process.inner_exclusive_access();
    let process_context_ptr = &mut process_inner.process_context as *mut TaskContext;
    // Change status to Ready
    process_inner.process_status = ProcessStatus::Ready;
    drop(process_inner);
    add_process(process);
    schedule(process_context_ptr);
}


/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_process_and_run_next(exit_code: i32) {
    // take from Processor
    let process = take_current_process().unwrap();
    let pid = process.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        if exit_code != 0 {
            //crate::sbi::shutdown(255); //255 == -1 for err hint
            shutdown(true)
        } else {
            //crate::sbi::shutdown(0); //0 for success hint
            shutdown(false)
        }
    }

    // **** access current TCB exclusively
    let mut process_inner = process.inner_exclusive_access();
    // Change status to Zombie
    process_inner.process_status = ProcessStatus::Zombie;
    // Record exit code
    process_inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in process_inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB

    process_inner.children.clear();
    // deallocate user space
    process_inner.address_space.recycle_data_pages();
    drop(process_inner);
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(process);
    // we do not have to save task context
    let mut _unused = TaskContext::new();
    schedule(&mut _unused as *mut _);
}

///Add init process to the manager
pub fn add_initproc() {
    add_process(INITPROC.clone());
}
