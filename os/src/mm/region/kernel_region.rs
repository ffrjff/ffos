#[allow(unused)]
use super::{MemoryRegion, Permission, PageTable, PTEFlags};
#[allow(unused)]
use super::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use crate::config::PAGE_SIZE;
use super::lazy_region::LazyRegion;

#[derive(Debug)]
pub struct KernelRegion {
    start: VirtPageNum,
    end: VirtPageNum,
    permission: Permission,
}

impl MemoryRegion for KernelRegion {
    fn map(&mut self, page_table: &mut PageTable) {
        println!("START:{}, END:{}", self.start.0, self.end.0);
        for num in self.start.0..self.end.0{
            let pte_flags = PTEFlags::from_bits(self.permission.bits()).unwrap();
            page_table.map(num.into(), num.into(), pte_flags);
        }
    }
    fn unmap(&mut self, page_table: &mut PageTable) {
        for num in self.start.0..self.end.0 {
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
        let mut start:usize = 0;
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
            let pte_flags = PTEFlags::from_bits(self.permission.bits()).unwrap();
            page_table.map(num.into(), num.into(), pte_flags);
        }
        self.end = new_end;
    }
    fn shrink(&mut self, page_table: &mut PageTable, new_end: VirtPageNum) {
        for num in self.end.0..new_end.0 {
            page_table.unmap(num.into());
        }
        self.end = new_end;
    }
    fn is_kernel_region(&self) -> Option<&KernelRegion> {
        Some(self)
    }
    fn is_lazy_region(&self) -> Option<&LazyRegion> {
        None
    }

}

impl KernelRegion {
    #[allow(unused)]
    pub fn new(start: VirtAddr, end: VirtAddr, permission: Permission) -> Self{
        let start_vpn = start.floor();
        let end_vpn = end.ceil();
        Self {
            start: start_vpn,
            end: end_vpn,
            permission,
        }

    }
}