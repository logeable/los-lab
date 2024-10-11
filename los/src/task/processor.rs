use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{error, mm, task::tcb::TaskStatus, trap::TrapContext};

use super::{
    manager::{self, fetch_from_runq, push_to_runq, switch_task},
    tcb::{TaskContext, TaskControlBlock},
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

    fn current_mut(&mut self) -> Option<&mut TaskControlBlock> {
        self.current.as_mut()
    }
}

pub fn take_current() -> TaskControlBlock {
    PROCESSOR
        .lock()
        .take_current()
        .expect("current tcb must exist")
}

pub fn run_tasks() -> ! {
    loop {
        let mut processor = PROCESSOR.lock();

        if let Some(mut next_tcb) = fetch_from_runq() {
            next_tcb.update_task_status(TaskStatus::Running);
            processor.current = Some(next_tcb);

            let idle_task_context = &mut processor.idle_task_context as *mut TaskContext;
            let next_task_context =
                &processor.current.as_ref().unwrap().context as *const TaskContext;

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
    let mut tcb = take_current();
    tcb.status = TaskStatus::Exited;
    schedule(&mut tcb.context);

    unreachable!();
}

pub fn suspend_current_task_and_schedule() {
    let mut tcb = take_current();
    tcb.status = TaskStatus::Ready;
    let task_context = push_to_runq(tcb);
    schedule(task_context);
}

pub fn get_current_task_name() -> Option<String> {
    PROCESSOR.lock().current_mut().map(|tcb| tcb.name.clone())
}

pub fn get_current_task_trap_context() -> Option<*mut TrapContext> {
    PROCESSOR
        .lock()
        .current_mut()
        .map(|tcb| tcb.get_trap_context_ptr())
}

pub fn get_current_task_satp() -> usize {
    PROCESSOR
        .lock()
        .current_mut()
        .map(|tcb| tcb.mem_space.page_table().satp())
        .expect("current task satp must exist")
}

pub fn fork_current_task() -> error::Result<usize> {
    let mut processor = PROCESSOR.lock();
    let current_tcb = processor.current_mut().expect("current tcb must exist");

    let forked_tcb = manager::fork_tcb(&current_tcb)?;
    let pid = forked_tcb.pid.pid();

    push_to_runq(forked_tcb);

    Ok(pid)
}

pub fn exec_in_tcb(path: &str) -> error::Result<()> {
    let tcb = PROCESSOR
        .lock()
        .current
        .as_mut()
        .map(|v| v as *mut TaskControlBlock)
        .expect("current tcb must exist");

    let tcb = unsafe { &mut *tcb };
    manager::load_elf_in_task(path, tcb)
}
