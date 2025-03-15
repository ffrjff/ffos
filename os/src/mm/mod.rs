pub mod heap_allocator;
pub mod frame_allocator;
pub mod address;
pub mod page_table;
pub mod address_space;
pub mod region;

#[allow(unused)]
pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
#[allow(unused)]
pub use frame_allocator::{frame_alloc, FrameTracker};
pub use page_table::PageTableEntry;
#[allow(unused)]
use page_table::PTEFlags;
pub use address_space::KERNEL_SPACE;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().apply_satp_and_flush_tlb();
}