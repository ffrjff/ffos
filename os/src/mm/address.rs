use super::PageTableEntry;
use crate::config::{PAGE_SIZE_BITS, PAGE_SIZE};
use core::fmt::{self, Debug, Formatter};

const PA_WIDTH_SV39: usize =56;
const PPN_WIDTH_SV39: usize = 44;




#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// struct of physical address
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// struct of virtual address
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// struct of physical address number
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// struct of virtual address number
pub struct VirtPageNum(pub usize);


/// Debug
impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}
impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}
impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
    }
}



/// transform
impl From<usize> for PhysAddr {
    fn from(value: usize) -> Self { 
        Self(value & ( (1 << PA_WIDTH_SV39) - 1 )) 
    }
}
impl From<PhysAddr> for usize {
    fn from(value: PhysAddr) -> Self { 
        value.0 
    }
}
impl From<PhysPageNum> for PhysAddr {
    fn from(value: PhysPageNum) -> Self { 
         Self(value.0 << PAGE_SIZE_BITS)
    }
}

impl From<usize> for PhysPageNum {
    fn from(value: usize) -> Self { 
        Self(value & ( (1 << PPN_WIDTH_SV39) - 1 )) 
    }
}
impl From<PhysPageNum> for usize {
    fn from(value: PhysPageNum) -> Self { 
        value.0 
    }
}
impl From<PhysAddr> for PhysPageNum {
    fn from(value: PhysAddr) -> Self {
        assert_eq!(value.page_offset(), 0);
        value.floor()
    }
}
impl PhysAddr {
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> PhysPageNum {
        if self.0 == 0 {
            PhysPageNum(0)
        } else {
            PhysPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
        }
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}
impl PhysPageNum {
    pub fn add(&mut self) {
        self.0 += 1;
    }
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512) }
    }
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }
}