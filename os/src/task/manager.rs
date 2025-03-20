use super::ProcessControlBlock;
use alloc::collections::VecDeque;
use crate::sync::UPSafeCell;
use alloc::sync::Arc;
use lazy_static::*;

pub struct ProcessManager {
    queue: VecDeque<Arc<ProcessControlBlock>>
}

lazy_static! {
    pub static ref PROCESS_MANAGER: UPSafeCell<ProcessManager> = unsafe {
        UPSafeCell::new(ProcessManager::new())
    };

}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
    pub fn enqueue(&mut self, process: Arc<ProcessControlBlock>) {
        self.queue.push_back(process);
    }
    pub fn dequeue(&mut self) -> Option<Arc<ProcessControlBlock>> {
        self.queue.pop_front()
    }
}

/// add process to process manager
pub fn add_process(process: Arc<ProcessControlBlock>) {
    PROCESS_MANAGER.exclusive_access().enqueue(process);
}

pub fn fetch_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESS_MANAGER.exclusive_access().dequeue()
}