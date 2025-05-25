use super::address::PhysPageNum;
use crate::mm::address::PhysAddr;
use alloc::vec::Vec;
use core::ops::Range;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref FRAME_ALLOCATOR: Mutex<StackFrameAllocator> = Mutex::new(StackFrameAllocator::new());
}

pub fn init(mem_range: &Range<usize>) {
    extern "C" {
        fn ekernel();
    }

    let start = PhysAddr::from(ekernel as usize);
    let end = PhysAddr::from(mem_range.end);
    assert!(
        start.0 < end.0,
        "no free frame, memory not enough, ekernel: {:#x}",
        ekernel as usize
    );

    FRAME_ALLOCATOR
        .lock()
        .init(start.ceil_ppn(), end.floor_ppn());
}

pub fn alloc() -> Option<Frame> {
    FRAME_ALLOCATOR.lock().alloc().map(Frame::new)
}

pub fn dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.lock().dealloc(ppn);
}

#[allow(dead_code)]
pub fn free_frames_count() -> usize {
    FRAME_ALLOCATOR.lock().free_frames_count()
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
            return Some(ppn.into());
        }

        if self.current == self.end {
            None
        } else {
            let ppn = self.current;
            self.current += 1;
            Some(ppn.into())
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        assert!(
            ppn.0 < self.current,
            "Frame ppn={:#x} has not been allocated!",
            ppn.0
        );
        assert!(
            !self.recycled.contains(&ppn.0),
            "Frame ppn={:#x} has been deallocated!",
            ppn.0
        );

        self.recycled.push(ppn.0);
    }

    pub fn free_frames_count(&self) -> usize {
        self.recycled.len() + self.end - self.current
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
