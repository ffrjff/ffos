use alloc::vec::Vec;
use lazy_static::*;
use super::address::{PhysAddr, PhysPageNum};
use crate::sync::UPSafeCell;
use crate::config::MEMORY_END;
use core::fmt::{self, Debug, Formatter};

/// manage a frame which has the same lifecycle as the tracker
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // page cleaning
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker: PPN={:#x}", self.ppn.0))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

#[allow(unused)]
pub struct FrameAllocator {
    current: PhysPageNum,
    end: PhysPageNum,
    recycled: Vec<PhysPageNum>,
}

impl FrameAllocator {
    pub fn new() -> Self {
        Self {
            current: PhysPageNum::from(0),
            end: PhysPageNum::from(0),
            recycled: Vec::new(),
        }
    }
    pub fn init(&mut self, low_addr: PhysPageNum, high_addr: PhysPageNum) {
        self.current = low_addr;
        self.end = high_addr;
    }
    pub fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            // println!("pop alloc: current: {}, end: {};alloced: {}", self.current.0, self.end.0, ppn.0);
            Some(ppn)
        } else {
            if self.current == self.end {
                None
            } else {
                self.current.add();
                // println!("add alloc: current: {}, end: {};alloced: {}", self.current.0, self.end.0, self.current.0-1);
                Some((self.current.0 - 1).into())
            }
        }
    }
    pub fn dealloc(&mut self, ppn: PhysPageNum) {
        // println!("dealloc: {}",ppn.0);
        if ppn >= self.current || 
            self.recycled
                .iter()
                .find(|&num| {*num == ppn})
                .is_some() {
                    // println!("dealloc: current: {}, end: {}", self.current.0, self.end.0);
                    panic!("PhysPageNum: {} has not been allocated!", ppn.0);
                }
        // println!("dealloc: {}",ppn.0);
        self.recycled.push(ppn);
    }
}

lazy_static! {
    /// frame allocator instance through lazy_static!
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocator> = unsafe {
        UPSafeCell::new(FrameAllocator::new())
    };
}

/// initiate the frame allocator using `ekernel` and `MEMORY_END`
pub fn init_frame_allocator() {
    extern  "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysPageNum::from(PhysAddr::from(ekernel as usize).ceil()),
        PhysPageNum::from(PhysAddr::from(MEMORY_END).floor()),
    );
}

/// allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(FrameTracker::new)
}

/// deallocate a frame
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}