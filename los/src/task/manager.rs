use super::{pid, TaskContext, TaskControlBlock};
use crate::{
    error,
    mm::{self, KernelStack},
    task::loader::AppLoader,
    trap::{trap_return, TrapContext},
};
use alloc::{collections::vec_deque::VecDeque, format, string::ToString, sync::Arc};
use core::arch::global_asm;
use lazy_static::lazy_static;
use spin::Mutex;

global_asm!(include_str!("switch.asm"));

lazy_static! {
    static ref TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}

pub struct TaskManager {
    runq: VecDeque<TaskControlBlock>,
    app_loader: AppLoader,
}

impl TaskManager {
    fn new() -> Self {
        let app_loader = AppLoader::load();

        TaskManager {
            runq: VecDeque::new(),
            app_loader,
        }
    }

    fn push_to_runq(&mut self, tcb: TaskControlBlock) {
        self.runq.push_back(tcb);
    }

    fn fetch_from_runq(&mut self) -> Option<TaskControlBlock> {
        self.runq.pop_front()
    }

    // fn find_next_ready_mut(&mut self) -> Option<(usize, &mut TaskControlBlock)> {
    //     let start_idx = match self.current_task_index {
    //         Some(current_idx) => current_idx + 1,
    //         None => 0,
    //     };

    //     let len = self.tasks.len();
    //     let mut found_idx = None;
    //     for i in start_idx..(start_idx + len) {
    //         let idx = i % len;
    //         let task = &mut self.tasks[idx];
    //         if task.status == TaskStatus::Ready {
    //             found_idx = Some(idx);
    //             break;
    //         }
    //     }

    //     found_idx.map(|idx| (idx, self.tasks.get_mut(idx).unwrap()))
    // }

    fn load_app(&self, name: &str) -> error::Result<TaskControlBlock> {
        let elf_data = self
            .app_loader
            .load_app_elf(name)
            .ok_or(error::KernelError::LoadAppELF(format!(
                "load app ELF failed: {}",
                name
            )))?;
        let (mem_space, user_sp, entry) =
            mm::build_app_mem_space(elf_data).expect("build app mem space must succeed");

        let trap_context_va = mm::trap_context_va();

        let trap_context_ppn = mem_space
            .page_table()
            .translate(trap_context_va.floor_vpn())
            .unwrap()
            .ppn();

        let pid = pid::alloc().ok_or(error::KernelError::AllocPid(
            "allocate pid failed".to_string(),
        ))?;
        let kernel_stack = KernelStack::map_in_kernel_memory_space(&pid)
            .expect("map app kernel stack must succeed");
        let kernel_stack_sp = kernel_stack.get_sp();
        let trap_context_dest = unsafe { trap_context_ppn.get_mut::<TrapContext>() };
        *trap_context_dest = TrapContext::init(entry, user_sp, kernel_stack_sp);

        Ok(TaskControlBlock::init(
            name.to_string(),
            pid,
            trap_return as usize,
            kernel_stack,
            mem_space,
        ))
    }
}

pub fn switch_task(current: *mut TaskContext, next: *const TaskContext) {
    extern "C" {
        fn _switch_task(current: *mut TaskContext, next: *const TaskContext);
    }

    unsafe { _switch_task(current, next) };
}

pub fn push_to_runq(tcb: TaskControlBlock) {
    TASK_MANAGER.lock().push_to_runq(tcb);
}

pub fn fetch_from_runq() -> Option<TaskControlBlock> {
    TASK_MANAGER.lock().fetch_from_runq()
}

pub fn create_tcb_by_app_name(name: &str) -> error::Result<TaskControlBlock> {
    TASK_MANAGER.lock().load_app(name)
}

pub fn list_apps() -> alloc::vec::Vec<alloc::string::String> {
    TASK_MANAGER.lock().app_loader.app_names()
}
