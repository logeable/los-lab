use super::{
    manager::{self, fetch_from_runq, push_to_runq, switch_task},
    tcb::{TaskContext, TaskControlBlockWrapper},
};
use crate::{error, task::tcb::TaskStatus, trap::TrapContext};
use alloc::format;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref PROCESSOR: Mutex<Processor> = Mutex::new(Processor::new());
}

struct Processor {
    current: Option<TaskControlBlockWrapper>,
    idle_task_context: TaskContext,
}

impl Processor {
    fn new() -> Self {
        Self {
            current: None,
            idle_task_context: TaskContext::EMPTY,
        }
    }

    fn take_current(&mut self) -> Option<TaskControlBlockWrapper> {
        self.current.take()
    }

    fn current(&mut self) -> Option<&TaskControlBlockWrapper> {
        self.current.as_ref()
    }
}

pub fn run_tasks() -> ! {
    loop {
        if let Some(next_tcb) = fetch_from_runq() {
            let (idle_task_context, next_task_context) = {
                let mut processor = PROCESSOR.lock();
                next_tcb.lock().update_task_status(TaskStatus::Running);

                let idle_task_context = &mut processor.idle_task_context as *mut TaskContext;
                let next_task_context = {
                    let mut next_tcb = next_tcb.lock();
                    &mut next_tcb.context as *const TaskContext
                };

                processor.current = Some(next_tcb);
                drop(processor);

                (idle_task_context, next_task_context)
            };

            switch_task(idle_task_context, next_task_context);
        }
    }
}

pub fn schedule(switched_task_context: *mut TaskContext) {
    let idle_task_context = {
        let processor = PROCESSOR.lock();
        &processor.idle_task_context as *const TaskContext
    };

    switch_task(switched_task_context, idle_task_context);
}

pub fn exit_current_task_and_schedule(exit_code: i32) -> ! {
    let current_tcb = PROCESSOR
        .lock()
        .take_current()
        .expect("current tcb must exist");

    {
        let init_proc_tcb = manager::get_init_proc_tcb();
        let mut current_tcb = current_tcb.lock();

        current_tcb.status = TaskStatus::Exited(exit_code);

        for child in current_tcb.children.iter() {
            child.lock().parent = Some(init_proc_tcb.clone());
            init_proc_tcb.lock().children.push(child.clone());
        }
    }

    // FIXME: release resources: memoryset, pid, kernel stack

    let task_context = {
        let mut tcb = current_tcb.lock();
        &mut tcb.context as *mut TaskContext
    };

    schedule(task_context);

    unreachable!();
}

pub fn suspend_current_task_and_schedule() {
    let tcb = PROCESSOR
        .lock()
        .take_current()
        .expect("current tcb must exist");

    tcb.lock().status = TaskStatus::Ready;
    let task_context = {
        let mut tcb = tcb.lock();
        &mut tcb.context as *mut TaskContext
    };

    push_to_runq(tcb.clone());

    schedule(task_context);
}

pub fn get_current_task_trap_context() -> Option<*mut TrapContext> {
    PROCESSOR
        .lock()
        .current()
        .map(|tcb| tcb.lock().get_trap_context_ptr())
}

pub fn get_current_task_satp() -> usize {
    PROCESSOR
        .lock()
        .current()
        .map(|tcb| tcb.lock().mem_space.page_table().satp())
        .expect("current task satp must exist")
}

pub fn fork_current_task() -> error::Result<usize> {
    let current_tcb = PROCESSOR
        .lock()
        .current()
        .expect("current tcb must exist")
        .clone();

    let forked_tcb = manager::fork_tcb(current_tcb)?;
    let pid = forked_tcb.lock().pid.pid();

    push_to_runq(forked_tcb);

    Ok(pid)
}

pub fn exec_in_tcb(path: &str) -> error::Result<()> {
    let tcb = PROCESSOR
        .lock()
        .current()
        .expect("current tcb must exist")
        .clone();

    manager::load_elf_in_task(path, tcb)
}

pub fn wait_child_exit(arg: WaitChildArg) -> error::Result<Option<ExitStatus>> {
    let exited_child = {
        let tcb = PROCESSOR
            .lock()
            .current()
            .expect("current tcb must exist")
            .clone();
        let mut tcb = tcb.lock();

        let index = match arg {
            WaitChildArg::Any => tcb.children.iter().position(|v| {
                let child_tcb = v.lock();
                matches!(child_tcb.status, TaskStatus::Exited(_))
            }),
            WaitChildArg::One(pid) => tcb.children.iter().position(|v| {
                let child_tcb = v.lock();
                child_tcb.pid.pid() == pid && matches!(child_tcb.status, TaskStatus::Exited(_))
            }),
        };

        index.map(|v| tcb.children.remove(v))
    };

    match exited_child {
        Some(exited_child) => {
            let exited_child = exited_child.lock();

            let exited_code = exited_child
                .status
                .get_exited_code()
                .expect("get exited code must succeed");

            return Ok(Some(ExitStatus {
                pid: exited_child.pid.pid(),
                exit_code: exited_code,
            }));
        }
        None => Ok(None),
    }
}

#[derive(Debug)]
pub enum WaitChildArg {
    Any,
    One(usize),
}

impl WaitChildArg {
    pub fn from_pid(pid: isize) -> error::Result<Self> {
        Ok(match pid {
            -1 => Self::Any,
            pid @ 1.. => Self::One(pid as usize),
            _ => return Err(error::KernelError::Common(format!("invalid pid: {}", pid))),
        })
    }
}

#[derive(Debug)]
pub struct ExitStatus {
    pub pid: usize,
    pub exit_code: i32,
}
