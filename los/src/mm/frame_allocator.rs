use crate::{config::MEMORY_END, mm::address::PhysAddr};

use super::address::PhysPageNum;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref FRAME_ALLOCATOR: Mutex<StackFrameAllocator> = Mutex::new(StackFrameAllocator::new());
}

pub fn init() {
    extern "C" {
        fn ekernel();
    }

    let start = PhysAddr::from(ekernel as usize);
    let end = PhysAddr::from(MEMORY_END);
    FRAME_ALLOCATOR
        .lock()
        .init(start.ceil_ppn(), end.floor_ppn());
}

pub fn alloc() -> Option<Frame> {
    FRAME_ALLOCATOR.lock().alloc().map(|ppn| Frame::new(ppn))
}

pub fn dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.lock().dealloc(ppn);
}

pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    pub fn init(&mut self, start: PhysPageNum, end: PhysPageNum) {
        self.current = start.0;
        self.end = end.0;
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else {
            if self.current == self.end {
                None
            } else {
                self.current += 1;
                Some((self.current - 1).into())
            }
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        if ppn.0 >= self.current {
            panic!("Frame ppn={:#x} has not been allocated!", ppn.0);
        }
        if self.recycled.iter().any(|&v| v == ppn.0) {
            panic!("Frame ppn={:#x} has been deallocated!", ppn.0);
        }

        self.recycled.push(ppn.0);
    }
}

#[derive(Debug)]
pub struct Frame {
    pub ppn: PhysPageNum,
}

impl Frame {
    fn new(mut ppn: PhysPageNum) -> Self {
        ppn.clear();
        Self { ppn }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        dealloc(self.ppn);
    }
}
