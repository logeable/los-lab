mod initproc;
mod loader;
mod manager;
mod pid;
mod processor;

use crate::mm::KernelStack;
use crate::mm::MemorySpace;
use crate::println;
use alloc::string::String;

use alloc::vec::Vec;
pub use pid::Pid;
pub use processor::exit_current_task_and_schedule;
pub use processor::get_current_task_name;
pub use processor::get_current_task_satp;
pub use processor::get_current_task_trap_context;
pub use processor::run_tasks;
pub use processor::suspend_current_task_and_schedule;
pub use processor::translate_by_current_task_pagetable;

pub fn init() {
    initproc::init();
}

#[derive(Debug)]
pub struct TaskControlBlock {
    pub name: String,
    pub pid: Pid,
    pub context: TaskContext,
    pub status: TaskStatus,
    pub kernel_stack: KernelStack,
    pub mem_space: MemorySpace,
}

impl TaskControlBlock {
    pub fn init(
        name: String,
        pid: Pid,
        ra: usize,
        kernel_stack: KernelStack,
        mem_space: MemorySpace,
    ) -> Self {
        Self {
            name,
            pid,
            context: TaskContext::init(ra, kernel_stack.get_sp()),
            status: TaskStatus::Ready,
            kernel_stack,
            mem_space,
        }
    }

    pub fn new(elf_data: &[u8]) -> Self {
        todo!()
    }

    pub fn exec(&self, elf_data: &[u8]) {}

    pub fn fork(&self) -> Self {
        todo!()
    }

    pub fn update_task_status(&mut self, status: TaskStatus) {
        self.status = status;
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
    pub const EMPTY: TaskContext = TaskContext::init(0, 0);

    const fn init(ra: usize, sp: usize) -> Self {
        Self { ra, sp, s: [0; 12] }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}

pub(crate) fn print_apps() {
    for (i, name) in manager::list_apps().iter().enumerate() {
        println!("{}: {}", i, name);
    }
}
