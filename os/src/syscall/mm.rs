use crate::mm::address_space::KERNEL_SPACE;
#[allow(unused)]
use crate::task::TASK_MANAGER;
use crate::config::PAGE_SIZE;
#[allow(unused)]
use crate::mm::address::{VirtAddr, PhysAddr, VirtPageNum, PhysPageNum};
#[allow(unused)]
use crate::mm::page_table::PTEFlags;
#[allow(unused)]
use crate::mm::frame_alloc;

#[allow(unused)]
pub fn sys_munmap(start: usize, len: usize) -> isize {
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    if len == 0 {
        return 0;
    }
    let end: VirtPageNum = VirtAddr::from(start + len).ceil();
    for num in VirtPageNum::from(start).0..end.0 {
        match KERNEL_SPACE.exclusive_access().page_table.find_pte(num.into()) {
            Some(_pte) => {
                KERNEL_SPACE.exclusive_access().page_table.unmap(num.into());
            }
            None => {
                return -1;
            }
        }
    }
    0

}

// pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {

// }