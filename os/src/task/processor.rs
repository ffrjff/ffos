use super::switch::__switch;
use super::task::ProcessStatus;
use super::{TaskContext, ProcessControlBlock};
use alloc::sync::Arc;
use lazy_static::*;
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use super::manager::fetch_process;

/// processor
pub struct Processor {
    current: Option<Arc<ProcessControlBlock>>,
    idle_process_context: TaskContext,
}


lazy_static! {
    /// init PROCESSOR
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

/// processor
impl Processor {
    /// used to create a new Processor
    pub fn new() -> Self {
        Self { current: None, idle_process_context: TaskContext::new() }
    }
    /// used to create a new Processor
    fn get_idle_process_context_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_process_context as *mut _
    }
    /// used to create a new Processor
    pub fn take_out_of_current_process(&mut self) -> Option<Arc<ProcessControlBlock>> {
        self.current.take()
    }
    /// used to create a new Processor
    pub fn clone_current_process(&mut self) -> Option<Arc<ProcessControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

/// run processes after kernel initiated
pub fn run_processes() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(process) = fetch_process() {
            let idle_process_context_ptr = processor.get_idle_process_context_ptr();
            let mut pcb_inner = process.inner_exclusive_access();
            let next_process_context_ptr = &pcb_inner.process_context as *const TaskContext;
            pcb_inner.process_status = ProcessStatus::Running;
            drop(pcb_inner);
            processor.current = Some(process);
            drop(processor);
            unsafe{
                __switch(idle_process_context_ptr, next_process_context_ptr);
            }
        }
    }
}

/// take out of current process and PROCESSOR current None
pub fn take_out_of_current_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESSOR.exclusive_access().take_out_of_current_process()
}
/// get current process
pub fn clone_current_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESSOR.exclusive_access().clone_current_process()
}
/// get token of the address space of current process
pub fn get_current_user_token() -> usize {
    let task = clone_current_process().unwrap();
    let token = task.inner_exclusive_access().get_user_token();
    token
}
/// get mut ref to trap context of current process
pub fn get_current_trap_context() -> &'static mut TrapContext {
    clone_current_process()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_context()
}

/// schedule: RR
pub fn schedule(switched_process_context_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_process_context_ptr();
    drop(processor);
    unsafe {
        __switch(switched_process_context_ptr, idle_task_cx_ptr);
    }
}