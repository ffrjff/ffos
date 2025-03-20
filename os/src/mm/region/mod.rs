use kernel_region::KernelRegion;
use lazy_region::LazyRegion;

#[allow(unused)]
use crate::mm::page_table::{PageTable, PTEFlags};
#[allow(unused)]
use crate::mm::address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use crate::mm::address_space::Permission;
use core::fmt;

mod file_region;
pub mod kernel_region;
pub mod lazy_region;
mod shared_region;

pub trait MemoryRegion: Send + Sync + fmt::Debug {
    #[allow(unused)]
    fn map(&mut self, page_table: &mut PageTable);
    #[allow(unused)]
    fn unmap(&mut self, page_table: &mut PageTable);
    fn copy_data(&mut self, page_table: &PageTable, data: &[u8]);
    fn extend(&mut self, page_table: &mut PageTable, new_end: VirtPageNum);
    fn shrink(&mut self, page_table: &mut PageTable, new_end: VirtPageNum);
    fn get_start(&self) -> VirtPageNum;
    fn get_end(&self) -> VirtPageNum;
    fn is_kernel_region(&self) -> Option<&KernelRegion>;
    fn is_lazy_region(&self) -> Option<&LazyRegion>;
    // fn is_shared_region(&self) -> bool;
    // fn is_file_region(&self) -> bool;
    // fn slipt();
    // pub fn fault_handler();
}


// pub trait ASRegion: Send + Sync {
//     fn metadata(&self) -> &ASRegionMeta;

//     fn metadata_mut(&mut self) -> &mut ASRegionMeta;

//     /// 将区域映射到页表，返回创建的页表帧
//     fn map(&self, root_pt: PageTable, overwrite: bool) -> Vec<HeapFrameTracker>;

//     /// 将区域取消映射到页表
//     fn unmap(&self, root_pt: PageTable);

//     /// 分割区域
//     fn split(&mut self, start: usize, size: usize) -> Vec<Box<dyn ASRegion>>;

//     /// 扩展区域
//     fn extend(&mut self, size: usize);

//     /// 拷贝区域
//     fn fork(&mut self, parent_pt: PageTable) -> Box<dyn ASRegion>;

//     /// 同步区域
//     fn sync(&self) {}

//     /// 错误处理
//     fn fault_handler(&mut self, root_pt: PageTable, vpn: VirtPageNum) -> SyscallResult<Vec<HeapFrameTracker>> {
//         Err(Errno::EINVAL)
//     }
// }
