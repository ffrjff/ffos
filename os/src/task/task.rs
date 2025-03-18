//! Types related to task management

use crate::mm::address_space::{AddressSpace, Permission};
#[allow(unused)]
use crate::mm::address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use crate::mm::KERNEL_SPACE;
use super::TaskContext;
use crate::trap::{trap_handler, TrapContext};
use crate::config::{kernel_stack_position, TRAP_CONTEXT};
use log::info;

#[derive(Debug)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_context: TaskContext,
    pub address_space: AddressSpace,
    pub trap_context_ppn: PhysPageNum,
    #[allow(unused)]
    pub base_size: usize,
    pub heap_bottom: usize,
    pub program_break: usize,
}
#[derive(Debug)]
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}

impl TaskControlBlock {
    /// init a TCB by app_id from it's elf data
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // println!("start to analyze app: {}", app_id);
        let (address_space, user_sp, entry_point) = AddressSpace::from_elf(elf_data);
        // println!("from_elf");
        let trap_context_ppn = 
            address_space
                .translate(VirtAddr::from(TRAP_CONTEXT).into())
                .unwrap()
                .ppn();
        let task_status = TaskStatus::Ready;
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE.exclusive_access().insert_lazy_framed_area_to_kernel(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            Permission::R | Permission::W,
        );
        let task_control_block = Self {
            task_status,
            task_context: TaskContext::init(kernel_stack_top),
            address_space,
            trap_context_ppn,
            base_size: user_sp,
            heap_bottom: user_sp,
            program_break: user_sp,
        };
        // prepare TrapContext in user space
        let trap_context = task_control_block.get_trap_context();
        *trap_context = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        println!("success to construct TCB of APP {}", app_id);
        info!("app {} cx: {:?}",app_id, task_control_block.task_context);
        task_control_block
    }
    pub fn get_trap_context(&self) -> &'static mut TrapContext {
        self.trap_context_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.address_space.token()
    }
    pub fn adjust_current_heap_size(&mut self, size: i32) -> Option<usize> {
        let old_break = self.program_break;
        let new_break = self.program_break as isize + size as isize;
        if new_break < self.heap_bottom as isize {
            return None;
        }
        let result = if size < 0 {
            self.address_space
                .decrease_heap_area(VirtAddr(self.heap_bottom), VirtAddr(new_break as usize))
        } else {
            self.address_space
                .increase_heap_area(VirtAddr(self.heap_bottom), VirtAddr(new_break as usize))
        };
        if result {
            self.program_break = new_break as usize;
            Some(old_break)
        } else {
            None
        }
    }
}