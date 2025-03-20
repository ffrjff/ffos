#[allow(unused)]
use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE};
#[allow(unused)]
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
#[allow(unused)]
use lazy_static::*;

lazy_static! {
    pub static ref PID_ALLOCATOR: UPSafeCell<PidAllocator> =
        unsafe { UPSafeCell::new(PidAllocator::new()) };
}

/// use to alloc pid
#[allow(unused)]
pub struct PidAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl PidAllocator {
    ///Create an empty `PidAllocator`
    pub fn new() -> Self {
        PidAllocator {
            current: 0,
            recycled: Vec::new(),
        }
    }
    #[allow(unused)]
    ///Allocate a pid
    pub fn alloc(&mut self) -> PidTracker {
        if let Some(pid) = self.recycled.pop() {
            PidTracker(pid)
        } else {
            self.current += 1;
            PidTracker(self.current - 1)
        }
    }
    #[allow(unused)]
    ///Recycle a pid
    pub fn dealloc(&mut self, pid: usize) {
        assert!(pid < self.current);
        assert!(
            !self.recycled.iter().any(|ppid| *ppid == pid),
            "pid {} has been deallocated!",
            pid
        );
        self.recycled.push(pid);
    }
}

#[derive(Debug)]
/// pid tracker
pub struct PidTracker(pub usize);

impl Drop for PidTracker {
    fn drop(&mut self) {
        //println!("drop pid {}", self.0);
        PID_ALLOCATOR.exclusive_access().dealloc(self.0);
    }
}
/// alloc pid
#[allow(unused)]
pub fn pid_alloc() -> PidTracker {
    PID_ALLOCATOR.exclusive_access().alloc()
}
