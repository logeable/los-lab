use alloc::string::{String, ToString};

use crate::{
    error,
    mm::{self, KernelStack, MemorySpace},
    trap::TrapContext,
};

use super::{manager, pid, Pid};

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

    pub fn get_trap_context_ptr(&self) -> *mut TrapContext {
        let trap_context_va = mm::trap_context_va();
        let trap_context_ppn = self
            .mem_space
            .page_table()
            .translate_vpn(trap_context_va.floor_vpn())
            .expect("translate must succeed")
            .ppn();
        let trap_context = unsafe { trap_context_ppn.get_mut::<TrapContext>() };

        trap_context
    }

    pub fn exec(&self, elf_data: &[u8]) {}

    pub fn update_task_status(&mut self, status: TaskStatus) {
        self.status = status;
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,
    pub s: [usize; 12],
}

impl TaskContext {
    pub const EMPTY: TaskContext = TaskContext::init(0, 0);

    pub const fn init(ra: usize, sp: usize) -> Self {
        Self { ra, sp, s: [0; 12] }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}
