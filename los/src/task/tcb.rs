use super::pid::Pid;
use crate::{
    mm::{self, KernelStack, MemorySpace},
    trap::TrapContext,
};
use alloc::{string::String, sync::Arc, vec::Vec};
use spin::Mutex;

pub type TaskControlBlockWrapper = Arc<Mutex<TaskControlBlock>>;

impl From<TaskControlBlock> for TaskControlBlockWrapper {
    fn from(value: TaskControlBlock) -> Self {
        Self::new(Mutex::new(value))
    }
}

#[derive(Debug)]
pub struct TaskControlBlock {
    pub name: String,
    pub pid: Pid,
    pub kernel_stack: KernelStack,
    pub mem_space: MemorySpace,
    pub context: TaskContext,
    pub status: TaskStatus,
    pub parent: Option<TaskControlBlockWrapper>,
    pub children: Vec<TaskControlBlockWrapper>,
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
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn get_trap_context_ptr(&self) -> *mut TrapContext {
        let trap_context_va = mm::trap_context_va();
        let mut trap_context_ppn = self
            .mem_space
            .page_table()
            .translate_vpn(trap_context_va.floor_vpn())
            .expect("translate must succeed")
            .ppn();
        let trap_context = unsafe { trap_context_ppn.get_mut::<TrapContext>() };

        trap_context
    }

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
    Exited(i32),
}

impl TaskStatus {
    pub fn get_exited_code(&self) -> Option<i32> {
        match self {
            TaskStatus::Exited(code) => Some(*code),
            _ => None,
        }
    }
}
