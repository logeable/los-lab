use core::ops::Deref;

use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::config::MAX_PID;

lazy_static! {
    static ref PID_ALLOCATOR: Mutex<PidAllocator> = Mutex::new(PidAllocator::new());
}

struct PidAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl PidAllocator {
    fn new() -> Self {
        Self {
            current: 1,
            end: MAX_PID,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<Pid> {
        if let Some(pid) = self.recycled.pop() {
            return Some(Pid { pid });
        }

        if self.current == self.end {
            None
        } else {
            let pid = self.current;
            self.current += 1;
            Some(Pid { pid })
        }
    }

    fn dealloc(&mut self, pid: usize) {
        assert!(pid < self.current, "Pid {pid} has not been allocated!");
        assert!(
            !self.recycled.contains(&pid),
            "Pid {pid} has been deallocated!"
        );
        self.recycled.push(pid);
    }
}

#[derive(Debug)]
pub struct Pid {
    pid: usize,
}

impl Pid {
    pub fn pid(&self) -> usize {
        self.pid
    }
}

impl Deref for Pid {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.pid
    }
}

impl Drop for Pid {
    fn drop(&mut self) {
        dealloc(self.pid);
    }
}

impl From<Pid> for usize {
    fn from(pid: Pid) -> Self {
        pid.pid
    }
}

pub fn alloc() -> Option<Pid> {
    PID_ALLOCATOR.lock().alloc()
}

pub fn dealloc(pid: usize) {
    PID_ALLOCATOR.lock().dealloc(pid);
}
