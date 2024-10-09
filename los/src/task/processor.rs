use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{error, mm, trap::TrapContext};

use super::{
    manager::{fetch_from_runq, push_to_runq, switch_task},
    TaskContext, TaskControlBlock, TaskStatus,
};

lazy_static! {
    static ref PROCESSOR: Mutex<Processor> = Mutex::new(Processor::new());
}

struct Processor {
    current: Option<TaskControlBlock>,
    idle_task_context: TaskContext,
}

impl Processor {
    fn new() -> Self {
        Self {
            current: None,
            idle_task_context: TaskContext::EMPTY,
        }
    }

    fn take_current(&mut self) -> Option<TaskControlBlock> {
        self.current.take()
    }

    fn current(&self) -> Option<&TaskControlBlock> {
        self.current.as_ref()
    }
}

pub fn take_current() -> Option<TaskControlBlock> {
    PROCESSOR.lock().take_current()
}

pub fn run_tasks() -> ! {
    loop {
        let mut processor = PROCESSOR.lock();

        if let Some(mut next_tcb) = fetch_from_runq() {
            next_tcb.update_task_status(TaskStatus::Running);
            let next_task_context = &next_tcb.context as *const TaskContext;
            let idle_task_context = &mut processor.idle_task_context as *mut TaskContext;

            processor.current = Some(next_tcb);

            drop(processor);

            switch_task(idle_task_context, next_task_context);
        }
    }
}

pub fn schedule(switched_task_context: *mut TaskContext) {
    let processor = PROCESSOR.lock();
    let idle_task_context = &processor.idle_task_context as *const TaskContext;

    drop(processor);

    switch_task(switched_task_context, idle_task_context);
}

pub fn exit_current_task_and_schedule() -> ! {
    let mut tcb = take_current().expect("current task must exist");
    tcb.status = TaskStatus::Exited;
    schedule(&mut tcb.context);

    unreachable!();
}

pub fn suspend_current_task_and_schedule() {
    let mut tcb = take_current().expect("current task must exist");
    tcb.status = TaskStatus::Ready;
    let task_context = &mut tcb.context as *mut TaskContext;
    push_to_runq(tcb);
    schedule(task_context);
}

pub fn get_current_task_name() -> Option<String> {
    PROCESSOR.lock().current().map(|v| v.name.clone())
}

pub fn get_current_task_trap_context() -> Option<*mut TrapContext> {
    let processor = PROCESSOR.lock();
    let tcb = processor.current().expect("current task must exist");
    let trap_context_va = mm::trap_context_va();
    let trap_context_ppn = tcb
        .mem_space
        .page_table()
        .translate(trap_context_va.floor_vpn())
        .unwrap()
        .ppn();
    let trap_context = unsafe { trap_context_ppn.get_mut::<TrapContext>() };

    Some(trap_context)
}

pub fn get_current_task_satp() -> Option<usize> {
    let processor = PROCESSOR.lock();
    processor.current().map(|v| v.mem_space.page_table().satp())
}

pub fn translate_by_current_task_pagetable(
    start_va: usize,
    len: usize,
) -> error::Result<Vec<&'static mut [u8]>> {
    let satp = get_current_task_satp().ok_or(error::KernelError::CurrentTaskNotFound(
        "current task not found".to_string(),
    ))?;
    mm::translate_by_satp(satp, start_va, len)
}
