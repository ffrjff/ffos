#[allow(unused)]
use alloc::vec;
#[allow(unused)]
use alloc::vec::Vec;
use bitflags::*;
use super::frame_allocator::frame_alloc;
use super::VirtPageNum;
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
pub struct PageTable {
    satp: PhysPageNum,
    frames: Vec<FrameTracker>,
}
impl PageTable {
    pub fn new() -> Self {
        let satp = frame_alloc().unwrap();
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

    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_or_create_pte(vpn).unwrap();
        // println!("vpn: {}, ppn: {}", vpn.0, ppn.0);
        assert!(!pte.valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    #[allow(unused)]
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
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
        for i in 0..3 {
            let pte_slice= unsafe { 
                core::slice::from_raw_parts_mut((PhysAddr::from(ppn)).0 as *mut PageTableEntry, 512) 
            };
            let pte = &mut pte_slice[indexes[i]];
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
    pub fn set_satp_flag(&self) -> usize {
        1usize << 63 | self.satp.0
    }
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