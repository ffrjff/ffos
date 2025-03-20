use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE};
use crate::task::PidTracker;
use super::address_space::Permission;
use super::KERNEL_SPACE;
use super::address::VirtAddr;

/// struct of user process's kernel stack position
#[allow(unused)]
#[derive(Debug)]
pub struct KernelStack {
    pid: usize,
}

/// return kernel stack's top and bottom(top > bottom)
#[allow(unused)]
pub fn kernel_stack_range(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}


#[allow(unused)]
impl KernelStack {
    /// create a new kernel stack by pid
    pub fn new(pid_tracker: &PidTracker) -> Self {
        let pid = pid_tracker.0;
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_range(pid);
        KERNEL_SPACE
            .exclusive_access()
            .insert_lazy_framed_area_to_kernel(
                kernel_stack_bottom.into(), 
                kernel_stack_top.into(), 
                Permission::R | Permission::W
            );
        KernelStack { pid }
    }

    // #[allow(unused)]
    // pub fn push_on_top<T>(&self, value: T) -> *mut T
    // where 
    //     T: Sized,
    // {
    //     let (_, kernel_stack_top) = kernel_stack_range(self.pid);
    //     let top_ptr = (kernel_stack_top - core::mem::size_of::<T>()) as *mut T;
    //     unsafe {
    //         *top_ptr = value;
    //     }
    //     top_ptr
    // } 
    pub fn get_top(&self) -> usize {
        let (_, kernel_stack_top) = kernel_stack_range(self.pid);
        kernel_stack_top
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let (kernel_stack_bottom, _) = kernel_stack_range(self.pid);
        let va: VirtAddr = kernel_stack_bottom.into();
        KERNEL_SPACE
            .exclusive_access()
            .region_delete_by_start(va.into());
    }
}