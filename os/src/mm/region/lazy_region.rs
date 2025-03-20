use crate::config::PAGE_SIZE;
use crate::mm::FrameTracker;
#[allow(unused)]
use crate::mm::frame_allocator::{frame_alloc, frame_dealloc};
#[allow(unused)]
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use super::KernelRegion;


#[allow(unused)]
use super::{MemoryRegion, PTEFlags, PageTable, Permission};
#[allow(unused)]
use super::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};

#[derive(Debug)]
pub struct LazyRegion {
    start: VirtPageNum,
    end: VirtPageNum,
    pages: BTreeMap<VirtPageNum, PageState>,
    permission: Permission,
}

#[derive(Debug)]
#[allow(unused)]
pub enum PageState {
    Free,
    Framed(FrameTracker),
    Cow(Arc<FrameTracker>),
}

impl MemoryRegion for LazyRegion {
    fn map(&mut self, page_table: &mut PageTable) {
        // println!("lazymap: START:{}, END:{}", self.start.0, self.end.0);
        for num in self.start.0..self.end.0 {
            let frame = frame_alloc().unwrap();
            let ppn = frame.ppn;
            // println!("vpn: {} is mapped to ppn: {}", num, ppn.0);
            self.pages.insert(num.into(), PageState::Framed(frame));
            let pte_flags = PTEFlags::from_bits(self.permission.bits()).unwrap();
            page_table.map(num.into(), ppn, pte_flags);
        }
    }
    fn unmap(&mut self, page_table: &mut PageTable) {
        for num in self.start.0..self.end.0 {
            self.pages.remove(&num.into());
            page_table.unmap(num.into());
        }
    }
    fn get_start(&self) -> VirtPageNum {
        self.start
    }
    fn get_end(&self) -> VirtPageNum {
        self.end
    }
    fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
        // println!("lazy copy_data");
        let mut start: usize = 0;
        let mut current_vpn = self.start;
        let len = data.len();
        let mut over: bool = false;
        loop {
            let mut end = start + PAGE_SIZE;
            if end >= len {
                end = len;
                over = true;
            }
            let src = &data[start..end];
            let dst = &mut page_table
                .trans_vpn_to_pte(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if over {
                break;
            }
            current_vpn.add();
        }
    }
    fn extend(&mut self, page_table: &mut PageTable, new_end: VirtPageNum) {
        for num in self.end.0..new_end.0 {
            let frame = frame_alloc().unwrap();
            let ppn = frame.ppn;
            self.pages.insert(num.into(), PageState::Framed(frame));
            let pte_flags = PTEFlags::from_bits(self.permission.bits()).unwrap();
            page_table.map(num.into(), ppn, pte_flags);
        }
        self.end = new_end;
    }
    fn shrink(&mut self, page_table: &mut PageTable, new_end: VirtPageNum) {
        for num in self.end.0..new_end.0 {
            self.pages.remove(&num.into());
            page_table.unmap(num.into());
        }
        self.end = new_end;
    }
    fn is_kernel_region(&self) -> Option<&KernelRegion> {
        None
    }
    fn is_lazy_region(&self) -> Option<&LazyRegion> {
        Some(self)
    }
}

impl LazyRegion {
    pub fn new(start: VirtAddr, end: VirtAddr, permission: Permission) -> Self {
        let start_vpn = start.floor();
        let end_vpn = end.ceil();
        Self {
            start: start_vpn,
            end: end_vpn,
            pages: BTreeMap::new(),
            permission,
        }
    }
    pub fn clone_region(region: &Self) -> Self {
        Self {
            start: region.get_start(),
            end: region.get_end(),
            pages: BTreeMap::new(),
            permission: region.permission,
        }
    }
    pub fn is_vpn_in_region(&self, vpn: VirtPageNum) -> bool {
        return (self.start.0 <= vpn.0) && (vpn.0 <= self.end.0)
    }
    pub fn map_one_cow_page(&self,) {

    }
    pub fn alloc_and_remap_cow_page() {

    }
}

// pub fn lazy_alloc(stval: VirtAddr) {

// }