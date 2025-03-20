//! Types related to task management

use crate::mm::address_space::AddressSpace;
#[allow(unused)]
use crate::mm::address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use crate::mm::kernel_space::KernelStack;
use crate::mm::KERNEL_SPACE;
use crate::sync::UPSafeCell;
use crate::task::pid::pid_alloc;
use super::TaskContext;
use crate::trap::{trap_handler, TrapContext};
use crate::config::TRAP_CONTEXT;
#[allow(unused)]
use log::info;
use alloc::vec::Vec;
use alloc::sync::{Arc, Weak};
use core::cell::RefMut;
use super::pid::PidTracker;

// #[derive(Debug)]
pub struct ProcessControlBlock {
    pub pid: Arc<PidTracker>,
    pub kernel_stack: KernelStack,
    inner: UPSafeCell<ProcessControlBlockInner>,
}

// #[derive(Debug)]
pub struct ProcessControlBlockInner {
    pub trap_context_ppn: PhysPageNum,
    pub base_size: usize,
    pub process_context: TaskContext,
    pub process_status: ProcessStatus,
    pub address_space: AddressSpace,
    pub parent: Option<Weak<ProcessControlBlock>>,
    pub children: Vec<Arc<ProcessControlBlock>>,
    pub exit_code: i32,
}


#[derive(Debug)]
#[derive(Copy, Clone, PartialEq)]
pub enum ProcessStatus {
    Ready,
    Running,
    Exited,
    Zombie,
}

impl ProcessControlBlock {
    pub fn inner_exclusive_access(&self) -> RefMut<'_, ProcessControlBlockInner> {
        self.inner.exclusive_access()
    }
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
        let pid_tracker = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_tracker);
        let kernel_stack_top = kernel_stack.get_top();
        let process_control_block = Self {
            pid: Arc::new(pid_tracker),
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(ProcessControlBlockInner {
                    trap_context_ppn,
                    base_size: user_sp,
                    process_context: TaskContext::init(kernel_stack_top),
                    process_status: ProcessStatus::Ready,
                    address_space,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                })
            },
        };
        // prepare TrapContext in user space
        let trap_context = process_control_block.inner_exclusive_access().get_trap_context();
        *trap_context = TrapContext::init_task_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        println!("success to construct TCB of APP {}", app_id);
        process_control_block
    }
    pub fn exec(&self, elf_data: &[u8]) {
        let (address_space, user_sp, entry_point) = AddressSpace::from_elf(elf_data);
        let trap_context_ppn = address_space
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        // **** access inner exclusively
        let mut inner = self.inner_exclusive_access();
        // substitute memory_set
        inner.address_space = address_space;
        // update trap_cx ppn
        inner.trap_context_ppn = trap_context_ppn;
        // initialize base_size
        inner.base_size = user_sp;
        // initialize trap_cx
        let trap_cx = inner.get_trap_context();
        *trap_cx = TrapContext::init_task_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
        );
        // **** release inner automatically
    }

    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // ---- access parent PCB exclusively
        let mut parent_inner = self.inner_exclusive_access();
        // copy user space(include trap context)
        let address_space = AddressSpace::from_existed_user(&parent_inner.address_space);
        let trap_context_ppn: PhysPageNum = address_space
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        let task_control_block = Arc::new(ProcessControlBlock {
            pid: Arc::new(pid_handle),
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(ProcessControlBlockInner {
                    trap_context_ppn,
                    base_size: parent_inner.base_size,
                    process_context: TaskContext::init(kernel_stack_top),
                    process_status: ProcessStatus::Ready,
                    address_space,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                })
            },
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        // modify kernel_sp in trap_cx
        // **** access children PCB exclusively
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_context();
        trap_cx.kernel_sp = kernel_stack_top;
        // return
        task_control_block
        // ---- release parent PCB automatically
        // **** release children PCB automatically
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
    // pub fn adjust_current_heap_size(&mut self, size: i32) -> Option<usize> {
    //     let old_break = self.program_break;
    //     let new_break = self.program_break as isize + size as isize;
    //     if new_break < self.heap_bottom as isize {
    //         return None;
    //     }
    //     let result = if size < 0 {
    //         self.address_space
    //             .decrease_heap_area(VirtAddr(self.heap_bottom), VirtAddr(new_break as usize))
    //     } else {
    //         self.address_space
    //             .increase_heap_area(VirtAddr(self.heap_bottom), VirtAddr(new_break as usize))
    //     };
    //     if result {
    //         self.program_break = new_break as usize;
    //         Some(old_break)
    //     } else {
    //         None
    //     }
    // }
}

impl ProcessControlBlockInner {
    pub fn get_trap_context(&self) -> &'static mut TrapContext {
        self.trap_context_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.address_space.token()
    }
    fn get_status(&self) -> ProcessStatus {
        self.process_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == ProcessStatus::Zombie
    }
}