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

#[allow(unused)]
use crate::mm::frame_alloc;
#[allow(unused)]
use crate::mm::page_table::PTEFlags;
#[allow(unused)]
use crate::config::PAGE_SIZE;
use crate::loader::{get_app_data, get_num_app};
#[allow(unused)]
use crate::mm::region::kernel_region::KernelRegion;
use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::vec::Vec;
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use log::info;
pub use context::TaskContext;
#[allow(unused)]
use riscv::register::satp;
#[allow(unused)]
use super::mm::address_space::KERNEL_SPACE;
#[allow(unused)]
use crate::mm::address::{VirtAddr, PhysAddr, VirtPageNum, PhysPageNum};

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    app_count: usize,
    /// use inner value to get mutable access
    task_queue: UPSafeCell<TaskQueue >,
}

/// Inner of Task Manageruse crate::mm::frame_alloc;
struct TaskQueue  {
    /// task list
    tasks: Vec<TaskControlBlock>,
    /// id of current `Running` task
    current_task: usize,
}

lazy_static! {
    /// TaskManager global instance
    pub static ref TASK_MANAGER: TaskManager = {
        info!("start to init TaskManager");
        let app_count = get_num_app();
        println!("num_app = {}", app_count);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..app_count {
            println!("load app: {}",i);
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
            info!("new load: {:?}", tasks[i]);
        }
        TaskManager {
            app_count,
            task_queue: unsafe {
                UPSafeCell::new(TaskQueue {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task.
    fn run_first_task(&self) -> ! {
        let mut task_queue = self.task_queue.exclusive_access();
        let first_task = &mut task_queue.tasks[0];
        // info!("app 0 cx: {:?}", next_task.task_context);
        first_task.task_status = TaskStatus::Running;
        let first_task_context_ptr = &first_task.task_context as *const TaskContext;
        // info!("kernel satp : {:#?}",satp::read());
        // match KERNEL_SPACE.exclusive_access().page_table.find_pte(0xffffffffffffe.into()) {
        //     Some(x)=> {
        //         info!("0xffffffffffffe: {:#x}",x.ppn().0);
        //     }
        //     None => {
        //         info!("ccccccccccccccc");
        //     }
        // }
        drop(task_queue);
        let mut fack_context = TaskContext::new();
        // info!("kernel satp : {:#?}",satp::read());
        // before this, we should drop local variables that must be dropped manually
        // println!("start to switch to 0");
        unsafe {
            __switch(&mut fack_context as *mut _, first_task_context_ptr);
        }
        panic!("error to run the first task");
    }
    // fn extend_current_task_memory(&self, start: usize, len: usize, prot: usize) -> isize{
    //     let mut task_queue = self.task_queue.exclusive_access();
    //     let current_task = task_queue.current_task;
    //     if start % PAGE_SIZE != 0 {
    //         return -1;
    //     }
    //     if prot & !0x7 != 0 {
    //         return -1;
    //     }
    //     if prot & 0x7 == 0 {
    //         return -1;
    //     }
    //     if len == 0 {
    //         return 0;
    //     }
    //     let end = ((len + start) / PAGE_SIZE + 1) * PAGE_SIZE;
    //     let mut permission = PTEFlags::empty();
    //     println!("start: {:#x}, end: {:#x} xxxxxxxxxxxxxxxxxx", start, end);
    //     if prot & 0b001 != 0 {
    //         permission |= PTEFlags::R;
    //     }
    //     if prot & 0b010 != 0 {
    //         permission |= PTEFlags::W;
    //     }
    //     if prot & 0b100 != 0 {
    //         permission |= PTEFlags::X;
    //     }
    //     let start_vpn:VirtPageNum = VirtAddr::from(start).into();
    //     let end_vpn:VirtPageNum = VirtAddr::from(end).into();
    //     println!("start: {:#?}, end: {:#?}", start_vpn, end_vpn);
    //     for num in start_vpn.0..end_vpn.0{
    //         match task_queue.tasks[current_task].address_space.page_table.find_pte(num.into()) {
    //             Some(_pte) => {
    //                 println!("vpn: {:#x} is already mapped to ppn: {:#x}", num, _pte.ppn().0);
    //                 return -1;
    //             }
    //             None => {
    //                 println!("vpn: {:#x} is not mapped", num);
    //                 match frame_alloc() {
    //                     Some(frame) => {
    //                         task_queue.tasks[current_task].address_space.page_table.map(num.into(), frame.ppn, permission);
    //                     }
    //                     None => {
    //                         println!("no enough space");
    //                         return -1;
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     0
    // }
    /// suspend the task running now
    fn suspend_current_task_running(&self) {
        let mut task_queue: core::cell::RefMut<'_, TaskQueue> = self.task_queue.exclusive_access();
        let current_task = task_queue.current_task;
        task_queue.tasks[current_task].task_status = TaskStatus::Ready;
    }

    /// exit the task running now
    fn exit_current_task_running(&self) {
        let mut task_queue = self.task_queue.exclusive_access();
        let current_task = task_queue.current_task;
        task_queue.tasks[current_task].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    fn find_next_ready_task(&self) -> Option<usize> {
        let task_queue = self.task_queue.exclusive_access();
        let current_task = task_queue.current_task;
        (current_task + 1..current_task + self.app_count + 1)
            .map(|id| id % self.app_count)
            .find(|id| task_queue.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Get the current 'Running' task's token.
    fn get_current_token(&self) -> usize {
        let task_queue = self.task_queue.exclusive_access();
        task_queue.tasks[task_queue.current_task].get_user_token()
    }

    /// Get the current 'Running' task's trap contexts.
    fn get_current_trap_context(&self) -> &'static mut TrapContext {
        let task_queue = self.task_queue.exclusive_access();
        task_queue.tasks[task_queue.current_task].get_trap_context()
    }

    /// Change the current 'Running' task's program break
    pub fn change_current_program_break(&self, size: i32) -> Option<usize> {
        let mut inner = self.task_queue.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].adjust_current_heap_size(size)
    }

    /// fin next task ready from task queue to run 
    fn run_next_task(&self) {
        if let Some(next_task) = self.find_next_ready_task() {
            let mut task_queue = self.task_queue.exclusive_access();
            let current_task = task_queue.current_task;
            task_queue.tasks[next_task].task_status = TaskStatus::Running;
            task_queue.current_task = next_task;
            let current_task_context_ptr = &mut task_queue.tasks[current_task].task_context as *mut TaskContext;
            let next_task_context_ptr = &task_queue.tasks[next_task].task_context as *const TaskContext;
            drop(task_queue);
            unsafe {
                __switch(current_task_context_ptr, next_task_context_ptr);
            }
        } else {
            println!("No application ready");
            shutdown(false);
        }
    }
}

/// run first task
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// rust next task
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// suspend current task
fn mark_current_suspended() {
    TASK_MANAGER.suspend_current_task_running();
}

/// exit current task
fn mark_current_exited() {
    TASK_MANAGER.exit_current_task_running();
}

/// suspend current task, then run next task
pub fn suspend_current_task_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// exit current task,  then run next task
pub fn exit_current_task_and_run_next() {
    mark_current_exited();
    run_next_task();
}
/// Get the current 'Running' task's token.
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

/// Get the current 'Running' task's trap contexts.
pub fn current_trap_context() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_context()
}

/// Change the current 'Running' task's program break
pub fn change_program_break(size: i32) -> Option<usize> {
    TASK_MANAGER.change_current_program_break(size)
}

// pub fn extend_current_task(start: usize, len: usize, prot: usize) -> isize{
//     TASK_MANAGER.extend_current_task_memory(start, len, prot)
// }
