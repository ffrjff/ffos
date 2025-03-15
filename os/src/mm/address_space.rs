#[allow(unused)]
use super::address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
#[allow(unused)]
use super::frame_allocator::{FrameTracker, frame_alloc};
use super::page_table::{PageTable, PTEFlags};
use super::region::MemoryRegion;
use crate::mm::region::kernel_region::KernelRegion;

use alloc::sync::Arc;
#[allow(unused)]
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::boxed::Box;

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

// /// manage address space
// #[allow(unused)]
// pub struct AddressSpace {
//     pid: usize,
//     pub page_table: PageTable,
//     memory_regions: Vec<MemoryRegion>,
//     entry_point: usize,
//     // kernel_info: Option<KernelAddressSpaceInfo>,
//     // trapcontext
//     // trampoline
// }

// /// manage range and quick add_map from va to pa
// #[allow(unused)]
// pub struct MemoryRegion {
//     range: Range<VirtPageNum>,
//     pages: BTreeMap<VirtPageNum, Arc<FrameTracker>>,
//     permission: Permission,
//     // map_type: MapType,
// }

#[allow(unused)]
pub struct AddressSpace {
    pid: usize,
    pub page_table: PageTable,
    memory_regions: Vec<Box<dyn MemoryRegion>>,
    entry_point: usize,
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

// pub struct MappedFile {
//   fd: usize,
//   start: usize,
//   size: usize,
//   offset: usize,
//   permissions: MemoryPermissions,
// }

// pub struct KernelAddressSpaceInfo {
//     metadata: usize,
//     reserved: [usize; 2],
// }

impl AddressSpace {
    pub fn new() -> Self {
        let new = Self {
            pid: 0,
            page_table: PageTable::new(),
            memory_regions: Vec::new(),
            entry_point: 0,
        };
        log::debug!("[AddressSpace::new()]");
        new
    }

    /// add a region to current AddrSpace
    pub fn region_add(&mut self, mut region: Box<dyn MemoryRegion>, data: Option<&[u8]>) {
        region.map(&mut self.page_table);
        if let Some(data) = data {
            region.copy_data(&self.page_table, data);
        }
        self.memory_regions.push(region);
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
        // map trampoline
        kernel_space.map_trampoline();
        // map kernel sections
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        println!("mapping .text section");
        println!("stext: {}, etext: {}", (stext as usize), (etext as usize));
        kernel_space.region_add(
            Box::new(KernelRegion::new(
                (stext as usize).into(),
                (etext as usize).into(),
                Permission::R | Permission::X,
            )),
            None,
        );
        println!("mapping .rodata section");
        kernel_space.region_add(
            Box::new(KernelRegion::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                Permission::R,
            )),
            None
        );
        println!("mapping .data section");
        kernel_space.region_add(
            Box::new(KernelRegion::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                Permission::R | Permission::W,
            )),
            None
        );
        println!("mapping .bss section");
        kernel_space.region_add(
            Box::new(KernelRegion::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                Permission::R | Permission::W,
            )),
            None
        );
        println!("mapping physical memory");
        kernel_space.region_add(
            Box::new(KernelRegion::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                Permission::R | Permission::W,
            )),
            None
        );
        println!("mapping memory-mapped registers");
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
    // pub fn region_add_framed(&mut self, start: VirtPageNum, end: VirtPageNum, permission: Permission, region_type: RegionType) {}
    // pub fn region_exetend() { }
    // pub fn region_refresh(&mut self, start: VirtPageNum, end: VirtPageNum, permission: Permission, region_type: RegionType) {}
    
    // /// create AddrSpace to kernel space
    // pub fn new_kernel() -> Self {
    //     let mut kernel_address_space = Self::new();
    //     kernel_address_space.
    // }
    // pub fn from_file();


    // #[allow(unused)]
    // pub fn from_elf(data: &[u8]) -> (Self, usize, usize) {
    //     let mut address_space = Self::new();
    //     address_space.map_trampoline();
    //     let elf = xmas_elf::ElfFile::new(data).unwrap();
    //     let elf_header = elf.header;
    //     assert_eq!(elf_header.pt1.magic, [0x07, 0x45, 0x4C, 0x46], "invalid elf file!");
    //     let ph_count = elf_header.pt2.ph_count();
    //     let mut max_end_vpn = VirtPageNum(0);
    //     for i in 0..ph_count {
    //         let ph = elf.program_header(i).unwrap();
    //         if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
    //             let start: VirtAddr= (ph.virtual_addr() as usize).into();
    //             let end: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
    //             let flags = ph.flags();
    //             let mut permission = Permission::U;
    //             if flags.is_read() {
    //                 permission |= Permission::R;
    //             }
    //             if flags.is_write() {
    //                 permission |= Permission::W;
    //             }
    //             if flags.is_execute() {
    //                 permission |= Permission::X;
    //             }
    //             let region =  MemoryRegion::new(start, end, permission);
    //             max_end_vpn = region.range.end;
    //             address_space.region_add(region, 
    //                 Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),);
    //         }
    //     }
    //     let max_end_va: VirtAddr = max_end_vpn.into();
    //     let mut user_stack_bottom: usize = max_end_va.into();
    //     // guard page
    //     user_stack_bottom += PAGE_SIZE;
    //     let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
    //     address_space.region_add(
    //         MemoryRegion::new(
    //             user_stack_bottom.into(),
    //             user_stack_top.into(),
    //             Permission::R | Permission::W | Permission::U,
    //         ),
    //         None,
    //     );
    //     // used in sbrk
    //     address_space.region_add(
    //         MemoryRegion::new(
    //             user_stack_top.into(),
    //             user_stack_top.into(),
    //             Permission::R | Permission::W | Permission::U,
    //         ),
    //         None,
    //     );
    //     // map TrapContext
    //     address_space.region_add(
    //         MemoryRegion::new(
    //             TRAP_CONTEXT.into(),
    //             TRAMPOLINE.into(),
    //             Permission::R | Permission::W,
    //         ),
    //         None,
    //     );
    //     (
    //         address_space,
    //         user_stack_top,
    //         elf.header.pt2.entry_point() as usize,
    //     )
    // }

    pub fn apply_satp_and_flush_tlb(&self) {
        let satp = self.page_table.set_satp_flag();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }
}

// impl MemoryRegion {
//     pub fn new(start: VirtAddr, end: VirtAddr, permission: Permission) -> Self {
//         let start_vpn: VirtPageNum = start.floor();
//         let end_vpn: VirtPageNum = end.floor();
//         Self {
//             range: start_vpn..end_vpn,
//             pages: BTreeMap::new(),
//             permission,
//         }
//     }

//     /// alloc physical region and add it to pagetable
//     pub fn add_to_pt(&mut self, pagetable: &mut PageTable) {
//         println!("START:{}, END:{}", self.range.start.0, self.range.end.0);
//         for num in self.range.start.0..self.range.end.0 {
//             let vpn = VirtPageNum(num);
//             self.add_one(pagetable, vpn);
//         }
//     }
//     /// dealloc 
//     #[allow(unused)]
//     pub fn delete_from_pt(&mut self, pagetable: &mut PageTable) {
//         for num in self.range.start.0..self.range.end.0 {
//             let vpn = VirtPageNum(num);
//             self.delete_one(pagetable, vpn);
//         } 
//     }
//     pub fn add_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
//         let frame = frame_alloc().unwrap();
//         let ppn = frame.ppn;
//         self.pages.insert(vpn, Arc::new(frame));
//         // self.pages.insert(vpn, frame);
//         page_table.map(vpn, ppn, PTEFlags::from_bits(self.permission.bits).unwrap());
//     }
//     pub fn delete_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
//         self.pages.remove(&vpn);
//         page_table.unmap(vpn);
//     }

//     pub fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
//         let mut start:usize = 0;
//         let mut current_vpn = self.range.start;
//         let len = data.len();
//         let mut over: bool = false;
//         loop {
//             let mut end = start + PAGE_SIZE;
//             if end >= len {
//                 end = len;
//                 over = true;
//             }
//             let src = &data[start..end];
//             let dst = &mut page_table
//                 .trans_vpn_to_pte(current_vpn)
//                 .unwrap()
//                 .ppn()
//                 .get_bytes_array()[..src.len()];
//             dst.copy_from_slice(src);
//             start += PAGE_SIZE;
//             if over {
//                 break;
//             }
//             current_vpn.add();
//         }
//     }




    // pub fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
    //     let mut start: usize = 0;
    //     let mut current_vpn = self.range.start;
    //     let len = data.len();
    //     loop {
    //         let src = &data[start..len.min(start + PAGE_SIZE)];
    //         let dst = &mut page_table
    //             .translate(current_vpn)
    //             .unwrap()
    //             .ppn()
    //             .get_bytes_array()[..src.len()];
    //         dst.copy_from_slice(src);
    //         start += PAGE_SIZE;
    //         if start >= len {
    //             break;
    //         }
    //         current_vpn.step();
    //     }
    // }
// }

// impl Debug for MemoryRegion {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        
//     }
// }

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