#[allow(unused)]
use alloc::vec;
#[allow(unused)]
use alloc::vec::Vec;
use bitflags::*;
use super::frame_allocator::frame_alloc;
use super::VirtPageNum;
use super::VirtAddr;
use alloc::string::String;
#[allow(unused)]
use super::{address::PhysPageNum, address::PhysAddr, FrameTracker};

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn:PhysPageNum, flags: PTEFlags) -> Self {
        Self {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    pub fn empty() -> Self {
        PageTableEntry {
            bits: 0,
        }
    }
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits((self.bits & ((1usize << 8) - 1)) as u8).expect("os/src/mm/page_table.rs PageTableEntry方法flags错误")
    }

    pub fn valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }
    pub fn readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }
    pub fn writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }
    pub fn executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }
    pub fn dirty(&self) -> bool {
        self.flags().contains(PTEFlags::D)
    }
}

/// struct of PageTable: only one member:satp(the basic addr of pt)
/// use: RA2 to equal lifetime
#[derive(Debug)]
pub struct PageTable {
    satp: PhysPageNum,
    frames: Vec<FrameTracker>,
}
#[allow(unused)]
use log::info;
impl PageTable {
    pub fn new() -> Self {
        let satp = frame_alloc().unwrap();
        // info!("kernel satp: {:#x}",satp.ppn.0);
        PageTable {
            satp: satp.ppn,
            frames: vec![satp],
        }
    }
    pub fn trans_vpn_to_pte(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }
    #[allow(unused)]
    pub fn trans_vpn_to_ppn(&self, vpn: VirtPageNum) -> Option<PhysPageNum> {
        Some(self.find_pte(vpn).map(|pte| *pte).unwrap().ppn())
    }
    pub fn trans_va_to_pa(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.find_pte(va.clone().floor()).map(|pte| {
            let aligned_pa: PhysAddr = pte.ppn().into();
            let offset = va.page_offset();
            let aligned_pa_usize: usize = aligned_pa.into();
            (aligned_pa_usize + offset).into()
        })
    }    
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_or_create_pte(vpn).unwrap();
        // println!("vpn: {}, ppn: {}", vpn.0, ppn.0);
        assert!(!pte.valid(), "vpn {:#x} is mapped before mapping", vpn.0);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    #[allow(unused)]
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.valid(), "vpn {:#x} is invalid before unmapping", vpn.0);
        *pte = PageTableEntry::empty();
    }
    pub fn from_token(satp: usize) -> Self {
        Self {
            satp: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }
    pub fn find_or_create_pte(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let mut ppn = self.satp;
        let indexes = vpn.get_pt_indexes();
        for i in 0..3 {
            let pte_slice= unsafe { 
                core::slice::from_raw_parts_mut((PhysAddr::from(ppn)).0 as *mut PageTableEntry, 512) 
            };
            let pte = &mut pte_slice[indexes[i]];
            if i == 2 {
                return Some(pte);
            }
            if !pte.valid() {
                // println!("create a new pt with index {}",i);
                let frame = frame_alloc().unwrap();
                // log::info!("vpn: {:#x} is alloc", vpn.0);
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
                // println!("new pte: {}",pte.ppn().0);
            }
            ppn = pte.ppn();
        }
        None
    }
    pub fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let mut ppn = self.satp;
        let indexes = vpn.get_pt_indexes();
        // println!("find_pte of {:#x}", vpn.0);
        // println!("satp: {:#x}",ppn.0);
        for i in 0..3 {
            let pte_slice= unsafe { 
                core::slice::from_raw_parts_mut((PhysAddr::from(ppn)).0 as *mut PageTableEntry, 512) 
            };
            let pte = &mut pte_slice[indexes[i]];
            // println!("{}, {:#x}",i, pte.ppn().0);
            if i == 2 {
                return Some(pte);
            }
            if !pte.valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        None
    }
    pub fn token(&self) -> usize {
        8usize << 60 | self.satp.0
    }
}

/// translate a pointer to a mutable u8 Vec through page table
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.trans_vpn_to_pte(vpn).unwrap().ppn();
        vpn.add();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}

pub fn translated_str(token: usize, ptr: *const u8) -> String {
    let page_table = PageTable::from_token(token);
    let mut string = String::new();
    let mut va = ptr as usize;
    loop {
        let ch: u8 = *(page_table
            .trans_va_to_pa(VirtAddr::from(va))
            .unwrap()
            .get_mut());
        if ch == 0 {
            break;
        } else {
            string.push(ch as char);
            va += 1;
        }
    }
    string
}

pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    //println!("into translated_refmut!");
    let page_table = PageTable::from_token(token);
    let va = ptr as usize;
    //println!("translated_refmut: before translate_va");
    page_table
        .trans_va_to_pa(VirtAddr::from(va))
        .unwrap()
        .get_mut()
}

#[allow(unused)]
pub fn pagetable_test() {
    let mut pagetable: PageTable = PageTable::new();
    let satp = pagetable.satp;
    pagetable.map(VirtPageNum(2), satp, PTEFlags::V);
    assert_eq!(pagetable.trans_vpn_to_pte(VirtPageNum(2)).unwrap().ppn(), satp);
    assert_eq!(pagetable.trans_vpn_to_ppn(VirtPageNum(2)).unwrap(), PhysPageNum::from(satp));
    match pagetable.find_pte(VirtPageNum(2)) {
        Some(x) => {println!("pagetable_test passed!");}
        None => {panic!("didn't find mapped virtual addr!");}
    }
    pagetable.map(VirtPageNum(3), satp, PTEFlags::V);
}