use crate::loader::get_app_data_by_name;
use crate::mm::page_table::{translated_str, translated_refmut};
use crate::task::{
    add_process, clone_current_process, get_current_user_token, exit_current_process_and_run_next,
    suspend_current_process_and_run_next,
};
use crate::timer::get_time_ms;
use alloc::sync::Arc;

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_process_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_process_and_run_next();
    0
}
pub fn sys_fork() -> isize {
    let current_process = clone_current_process().unwrap();
    let new_process = current_process.fork();
    let new_pid = new_process.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_process.inner_exclusive_access().get_trap_context();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_process(new_process);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = get_current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = clone_current_process().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

pub fn sys_getpid() -> isize {
    clone_current_process().unwrap().pid.0 as isize
}

/// get time in milliseconds
pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let process = clone_current_process().unwrap();
    // find a child process

    // ---- access current TCB exclusively
    let mut process_inner = process.inner_exclusive_access();
    if !process_inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = process_inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB lock exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = process_inner.children.remove(idx);
        // confirm that child will be deallocated after removing from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child TCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(process_inner.address_space.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB lock automatically
}
