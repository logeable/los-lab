use super::{TaskContext, TaskControlBlock, TaskStatus};
use crate::{
    error,
    mm::{self, address},
    task::loader::AppLoader,
    trap::{trap_return, TrapContext},
};
use alloc::{format, vec::Vec};
use core::arch::global_asm;
use lazy_static::lazy_static;
use spin::Mutex;

global_asm!(include_str!("switch.asm"));

const KERNEL_STACK_SIZE: usize = 4096 * 4;

lazy_static! {
    pub static ref TASK_MANAGER: Mutex<TaskManager> = {
        let manager = TaskManager::new();

        Mutex::new(manager)
    };
}

pub struct TaskManager {
    tasks: Vec<TaskControlBlock>,
    current_task_index: Option<usize>,
}

impl TaskManager {
    fn new() -> Self {
        let app_loader = AppLoader::load();
        let mut tasks = Vec::new();
        for (i, app) in app_loader.apps().iter().enumerate() {
            let (mem_space, user_sp, entry) =
                mm::build_app_mem_space(app.elf_data).expect("build app mem space must succeed");

            let trap_context_va = mm::trap_context_va();

            let trap_context_ppn = mem_space
                .page_table()
                .translate(trap_context_va.floor_vpn())
                .unwrap()
                .ppn();
            let kernel_sp = mm::add_app_kernel_stack_area_in_kernel_space(i)
                .expect("map app kernel stack must succeed");
            let trap_context_dest = unsafe { trap_context_ppn.get_mut::<TrapContext>() };
            *trap_context_dest = TrapContext::init(entry, user_sp, kernel_sp);
            tasks.push(TaskControlBlock::init(
                app.name,
                trap_return as usize,
                kernel_sp,
                mem_space,
            ));
        }
        TaskManager {
            tasks,
            current_task_index: None,
        }
    }

    fn find_next_ready_mut(&mut self) -> Option<(usize, &mut TaskControlBlock)> {
        let start_idx = match self.current_task_index {
            Some(current_idx) => current_idx + 1,
            None => 0,
        };

        let len = self.tasks.len();
        let mut found_idx = None;
        for i in start_idx..(start_idx + len) {
            let idx = i % len;
            let task = &mut self.tasks[idx];
            if task.status == TaskStatus::Ready {
                found_idx = Some(idx);
                break;
            }
        }

        found_idx.map(|idx| (idx, self.tasks.get_mut(idx).unwrap()))
    }

    fn get_current_mut(&mut self) -> Option<&mut TaskControlBlock> {
        match self.current_task_index {
            Some(idx) => self.tasks.get_mut(idx),
            None => None,
        }
    }

    fn set_current(&mut self, idx: usize) {
        self.current_task_index = Some(idx);
    }
}

pub fn switch_task(current: *mut TaskContext, next: *const TaskContext) {
    extern "C" {
        fn _switch_task(current: *mut TaskContext, next: *const TaskContext);
    }

    unsafe { _switch_task(current, next) };
}

pub fn schedule() {
    let mut unused = TaskContext::init(0, 0);
    let current_context = {
        TASK_MANAGER
            .lock()
            .get_current_mut()
            .map(|v| &mut v.context)
            .unwrap_or(&mut unused) as *mut TaskContext
    };

    let next_context = {
        let mut task_manager = TASK_MANAGER.lock();
        let (next_idx, next_task) = task_manager
            .find_next_ready_mut()
            .expect("no more tasks to schedule");

        let next_context: *const TaskContext = &next_task.context;

        next_task.status = TaskStatus::Running;
        task_manager.set_current(next_idx);

        next_context
    };

    switch_task(current_context, next_context);
}

pub fn exit_current_task_and_schedule() -> ! {
    TASK_MANAGER.lock().get_current_mut().unwrap().status = TaskStatus::Exited;
    schedule();

    unreachable!();
}

pub fn suspend_current_task_and_schedule() {
    TASK_MANAGER.lock().get_current_mut().unwrap().status = TaskStatus::Ready;
    schedule();
}

pub fn get_current_task_name() -> Option<&'static str> {
    TASK_MANAGER.lock().get_current_mut().map(|tcp| tcp.name)
}

pub fn get_current_task_mut() -> Option<*mut TaskControlBlock> {
    TASK_MANAGER
        .lock()
        .get_current_mut()
        .map(|v| v as *mut TaskControlBlock)
}

pub fn get_current_task_trap_context() -> Option<*mut TrapContext> {
    let tcb = match get_current_task_mut() {
        Some(tcb) => unsafe { &*tcb },
        None => return None,
    };
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

pub fn translate_by_current_task_pagetable(va: usize) -> error::Result<usize> {
    let vpn = address::VirtAddr::from(va).floor_vpn();

    let name = get_current_task_name().expect("current task must exist");
    match TASK_MANAGER
        .lock()
        .get_current_mut()
        .expect("current task must exist")
        .mem_space
        .page_table()
        .translate(vpn)
    {
        Some(pte) => {
            let pa = address::PhysAddr::from(pte.ppn());
            return Ok(usize::from(pa) + (va % address::PAGE_SIZE));
        }
        None => {
            return Err(error::KernelError::Translate(format!(
                "translate address {:#x} for user space {:?} failed",
                va, name
            )));
        }
    }
}
