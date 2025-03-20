#[allow(unused)]
use super::address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
#[allow(unused)]
use super::frame_allocator::{FrameTracker, frame_alloc};
use super::page_table::{PageTable, PTEFlags, PageTableEntry};
use super::region::MemoryRegion;
use crate::mm::region::kernel_region::KernelRegion;
use crate::mm::region::lazy_region::LazyRegion;

use alloc::sync::Arc;
#[allow(unused)]
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::boxed::Box;
use log::info;

use core::arch::asm;
#[allow(unused)]
use core::fmt::Debug;
#[allow(unused)]
use core::ops::Range;

use bitflags::bitflags;
use riscv::register::satp;
use lazy_static::*;
#[allow(unused)]
use crate::config::{PAGE_SIZE, TRAMPOLINE, USER_STACK_SIZE, TRAP_CONTEXT, MEMORY_END, MMIO};
use crate::sync::UPSafeCell;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

lazy_static! {
    /// a memory set instance through lazy_static! managing kernel space
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<AddressSpace>> =
        Arc::new(unsafe { UPSafeCell::new(AddressSpace::new_kernel()) });
}

#[derive(Debug)]
#[allow(unused)]
pub struct AddressSpace {
    pub page_table: PageTable,
    memory_regions: Vec<Box<dyn MemoryRegion>>,
    // region_map: BTreeMap<VirtPageNum, usize>,
    // kernel_info: Option<KernelAddressSpaceInfo>,
}



bitflags! {
    pub struct Permission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

impl AddressSpace {
    pub fn new() -> Self {
        let new = Self {
            page_table: PageTable::new(),
            memory_regions: Vec::new(),
        };
        log::debug!("[AddressSpace::new()]");
        new
    }
    pub fn token(&self) -> usize {
        self.page_table.token()
    }
    /// add a region to current AddrSpace
    pub fn region_add(&mut self, mut region: Box<dyn MemoryRegion>, data: Option<&[u8]>) {
        region.map(&mut self.page_table);
        // println!("after map");
        if let Some(data) = data {
            region.copy_data(&self.page_table, data);
        }
        self.memory_regions.push(region);
    }
    pub fn region_delete_by_start(&mut self, start: VirtPageNum) {
        if let Some((idx, area)) = self
            .memory_regions
            .iter_mut()
            .enumerate()
            .find(|(_, area)| area.get_start() == start)
        {
            area.unmap(&mut self.page_table);
            self.memory_regions.remove(idx);
        }
    }
    pub fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }
    pub fn new_kernel() -> Self {
        let mut kernel_space = AddressSpace::new();
        kernel_space.map_trampoline();
        info!("trampoline is mapped to kernel space");
        info!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        info!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        info!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        info!(".bss [{:#x}, {:#x})", sbss_with_stack as usize, ebss as usize);
        info!("mapping .text section");
        // println!("stext: {}, etext: {}", (stext as usize), (etext as usize));
        kernel_space.region_add(
            Box::new(KernelRegion::new(
                (stext as usize).into(),
                (etext as usize).into(),
                Permission::R | Permission::X,
            )),
            None,
        );
        info!("mapping .rodata section");
        kernel_space.region_add(
            Box::new(KernelRegion::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                Permission::R,
            )),
            None
        );
        info!("mapping .data section");
        kernel_space.region_add(
            Box::new(KernelRegion::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                Permission::R | Permission::W,
            )),
            None
        );
        info!("mapping .bss section");
        kernel_space.region_add(
            Box::new(KernelRegion::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                Permission::R | Permission::W,
            )),
            None
        );
        info!("mapping physical memory");
        kernel_space.region_add(
            Box::new(KernelRegion::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                Permission::R | Permission::W,
            )),
            None
        );
        info!("mapping memory-mapped registers");
        for pair in MMIO {
            kernel_space.region_add(
                Box::new(KernelRegion::new(
                    (*pair).0.into(),
                    ((*pair).0 + (*pair).1).into(),
                    Permission::R | Permission::W,
                )),
                None
            );
        }
        kernel_space
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.trans_vpn_to_pte(vpn)
    }
    // pub fn region_add_framed(&mut self, start: VirtPageNum, end: VirtPageNum, permission: Permission, region_type: RegionType) {}
    // pub fn region_exetend() { }
    // pub fn region_refresh(&mut self, start: VirtPageNum, end: VirtPageNum, permission: Permission, region_type: RegionType) {}
    
    // /// create AddrSpace to kernel space
    // pub fn from_file();


    // #[allow(unused)]
    pub fn from_elf(data: &[u8]) -> (Self, usize, usize) {
        let mut address_space = Self::new();
        address_space.map_trampoline();
        let elf = xmas_elf::ElfFile::new(data).unwrap();
        let elf_header = elf.header;
        assert_eq!(elf_header.pt1.magic, [0x7f, 0x45, 0x4C, 0x46], "invalid elf file!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        // println!("entry: {}",elf.header.pt2.entry_point() as usize);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            // println!("ph_count: {}, LOAD? :{}", i, ph.get_type().unwrap() == xmas_elf::program::Type::Load );
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start: VirtAddr= (ph.virtual_addr() as usize).into();
                let end: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let flags = ph.flags();
                let mut permission = Permission::U;
                if flags.is_read() {
                    permission |= Permission::R;
                }
                if flags.is_write() {
                    permission |= Permission::W;
                }
                if flags.is_execute() {
                    permission |= Permission::X;
                }
                let region =  LazyRegion::new(start, end, permission);
                max_end_vpn = region.get_end();
                address_space.region_add(
                    Box::new(region)
                    , Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),);
            }
        }
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        address_space.region_add(
            Box::new(LazyRegion::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                Permission::R | Permission::W | Permission::U,
            )),
            None,
        );
        // used in sbrk
        address_space.region_add(
            Box::new(LazyRegion::new(
                user_stack_top.into(),
                user_stack_top.into(),
                Permission::R | Permission::W | Permission::U,
            )),
            None,
        );
        // map TrapContext
        address_space.region_add(
            Box::new(LazyRegion::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                Permission::R | Permission::W,
            )),
            None,
        );
        (
            address_space,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }

    pub fn from_existed_user(user_space: &Self) -> Self {
        let mut address_space = Self:: new();
        address_space.map_trampoline();
        let mut new_region: LazyRegion;
        for region in user_space.memory_regions.iter() {
            if let Some(lazy_region) = region.is_lazy_region() {
                new_region = LazyRegion::clone_region(lazy_region);
                address_space.region_add(Box::new(new_region), None);
                for num in region.get_start().0..region.get_end().0 {
                    let src_ppn = user_space.translate(num.into()).unwrap().ppn();
                    let dst_ppn = address_space.translate(num.into()).unwrap().ppn();
                    dst_ppn
                        .get_bytes_array()
                        .copy_from_slice(src_ppn.get_bytes_array());
                }
            } else {
                panic!("Region is not of type LazyRegion");
            }
        }
        address_space
    }

    pub fn apply_satp_and_flush_tlb(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }
    pub fn recycle_data_pages(&mut self) {
        //*self = Self::new_bare();
        self.memory_regions.clear();
    }
    pub fn insert_lazy_framed_area_to_kernel(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: Permission
    ) {
        self.region_add(
            Box::new(LazyRegion::new(start_va, end_va, permission)),
            None,
        );
    }
    #[allow(unused)]
    pub fn decrease_heap_area(&mut self, start: VirtAddr, new_end: VirtAddr) -> bool {
        if let Some(region) = self
            .memory_regions
            .iter_mut()
            .find(|region| region.get_start() == start.floor())
        {
            region.shrink(&mut self.page_table, new_end.ceil());
            true
        } else {
            false
        }
    }
    #[allow(unused)]
    pub fn increase_heap_area(&mut self, start: VirtAddr, new_end: VirtAddr) -> bool {
        if let Some(region) = self
            .memory_regions
            .iter_mut()
            .find(|region| region.get_start() == start.floor())
        {
            region.extend(&mut self.page_table, new_end.ceil());
            true
        } else {
            false
        }
    }
}








































#[allow(unused)]
pub fn remap_test() {
    let mut kernel_space = KERNEL_SPACE.exclusive_access();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert!(!kernel_space
        .page_table
        .trans_vpn_to_pte(mid_text.floor())
        .unwrap()
        .writable(),);
    assert!(!kernel_space
        .page_table
        .trans_vpn_to_pte(mid_rodata.floor())
        .unwrap()
        .writable(),);
    assert!(!kernel_space
        .page_table
        .trans_vpn_to_pte(mid_data.floor())
        .unwrap()
        .executable(),);
    println!("remap_test passed!");
}