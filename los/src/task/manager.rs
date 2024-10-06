use super::{TaskContext, TaskControlBlock, TaskStatus};
use crate::{
    error,
    mm::{
        self,
        address::{self},
        PageTable,
    },
    task::loader::AppLoader,
    trap::{trap_return, TrapContext},
};
use alloc::{string::ToString, vec::Vec};
use core::arch::global_asm;
use lazy_static::lazy_static;
use spin::Mutex;

global_asm!(include_str!("switch.asm"));

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

pub fn translate_by_current_task_pagetable(
    start_va: usize,
    len: usize,
) -> error::Result<Vec<&'static mut [u8]>> {
    let satp = TASK_MANAGER
        .lock()
        .get_current_mut()
        .expect("current task must exist")
        .mem_space
        .page_table()
        .satp();

    translate_by_satp(satp, start_va, len)
}

pub fn translate_by_satp(
    satp: usize,
    start_va: usize,
    len: usize,
) -> error::Result<Vec<&'static mut [u8]>> {
    let page_table = PageTable::from_satp(satp);
    let end_va = start_va + len;

    let mut result: Vec<&'static mut [u8]> = Vec::new();

    let mut addr = start_va;
    while addr < end_va {
        let addr_va = address::VirtAddr::from(addr);

        let page_vpn = addr_va.floor_vpn();
        let page_ppn = page_table
            .translate(page_vpn)
            .ok_or(error::KernelError::Translate(
                "translate start vpn failed".to_string(),
            ))?
            .ppn();

        let page_start_va = address::VirtAddr::from(page_vpn);
        let page_end_va = address::VirtAddr::from(page_vpn.offset(1));

        let chunk_end_va = page_end_va.0.min(end_va);
        let chunk_start_va = page_start_va.0.max(addr_va.0);
        let chunk_page_offset = addr_va.0 - page_start_va.0;
        let chunk_len = chunk_end_va - chunk_start_va;

        let page_start_pa = address::PhysAddr::from(page_ppn);
        let chunk_start_pa = page_start_pa.0 + chunk_page_offset;
        let chunk =
            unsafe { core::slice::from_raw_parts_mut(chunk_start_pa as *mut u8, chunk_len) };

        addr += chunk_len;

        result.push(chunk);
    }

    Ok(result)
}
