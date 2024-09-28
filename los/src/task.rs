mod loader;
mod manager;

pub use manager::exit_current_task_and_schedule;
pub use manager::get_current_task_mut;
pub use manager::get_current_task_name;
pub use manager::get_current_task_trap_context;
pub use manager::schedule;
pub use manager::suspend_current_task_and_schedule;
pub use manager::translate_by_current_task_pagetable;

use crate::mm::MemorySpace;

#[derive(Debug)]
pub struct TaskControlBlock {
    pub name: &'static str,
    pub context: TaskContext,
    pub status: TaskStatus,
    pub mem_space: MemorySpace,
}

impl TaskControlBlock {
    pub fn init(name: &'static str, ra: usize, sp: usize, mem_space: MemorySpace) -> Self {
        Self {
            name,
            context: TaskContext::init(ra, sp),
            status: TaskStatus::Ready,
            mem_space,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,
    pub s: [usize; 12],
}

impl TaskContext {
    fn init(ra: usize, sp: usize) -> Self {
        Self { ra, sp, s: [0; 12] }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}
