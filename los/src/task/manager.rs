use super::{
    pid,
    tcb::{TaskContext, TaskControlBlock, TaskStatus},
};
use crate::{
    error,
    mm::{self, KernelStack, MemorySpace},
    println,
    task::loader::AppLoader,
    trap::{trap_return, TrapContext},
};
use alloc::{collections::vec_deque::VecDeque, format, string::ToString};
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

    fn push_to_runq(&mut self, tcb: TaskControlBlock) -> *mut TaskContext {
        self.runq.push_back(tcb);
        self.runq.back_mut().map(|v| &mut v.context).unwrap()
    }

    fn fetch_from_runq(&mut self) -> Option<TaskControlBlock> {
        self.runq.pop_front()
    }

    fn create_task(&self, name: &str) -> error::Result<TaskControlBlock> {
        let elf_data = self
            .app_loader
            .load_app_elf(name)
            .ok_or(error::KernelError::LoadAppELF(format!(
                "load app ELF failed: {}",
                name
            )))?;
        let (mut mem_space, user_sp, entry) =
            mm::build_app_mem_space(elf_data).expect("build app mem space must succeed");

        let pid = pid::alloc().ok_or(error::KernelError::AllocPid(
            "allocate pid failed".to_string(),
        ))?;
        let kernel_stack = KernelStack::map_in_kernel_memory_space(&pid)
            .expect("map app kernel stack must succeed");
        let kernel_stack_sp = kernel_stack.get_sp();

        let trap_context = TrapContext::init(entry, user_sp, kernel_stack_sp);
        let trap_context_dest = mem_space.trap_context_mut_ptr();
        unsafe { *trap_context_dest = trap_context };

        Ok(TaskControlBlock::init(
            name.to_string(),
            pid,
            trap_return as usize,
            kernel_stack,
            mem_space,
        ))
    }

    fn fork_task(&self, tcb: &TaskControlBlock) -> error::Result<TaskControlBlock> {
        let pid = pid::alloc().ok_or(error::KernelError::AllocPid(
            "allocate pid failed".to_string(),
        ))?;
        let name = tcb.name.clone();
        let kernel_stack = KernelStack::map_in_kernel_memory_space(&pid)
            .expect("map app kernel stack must succeed");
        let context = TaskContext::init(trap_return as usize, kernel_stack.get_sp());

        let mut mem_space = tcb.mem_space.fork().map_err(|e| {
            error::KernelError::Common(format!("fork memory space failed: {:?}", e))
        })?;

        let current_trap_context = unsafe { (&*tcb.get_trap_context_ptr()).clone() };
        let mut trap_context = TrapContext {
            kernel_sp: kernel_stack.get_sp(),
            ..current_trap_context
        };
        trap_context.regs[10] = 0;
        trap_context.sepc += 4;
        let trap_context_dest = mem_space.trap_context_mut_ptr();
        unsafe { *trap_context_dest = trap_context };

        let tcb = TaskControlBlock {
            name,
            pid,
            context,
            status: TaskStatus::Ready,
            kernel_stack,
            mem_space,
        };

        Ok(tcb)
    }

    fn load_elf_in_task(&self, path: &str, tcb: &mut TaskControlBlock) -> error::Result<()> {
        let elf_data = self
            .app_loader
            .load_app_elf(path)
            .ok_or(error::KernelError::LoadAppELF(format!(
                "load app ELF failed: {}",
                path
            )))?;
        let (mut mem_space, user_sp, entry) =
            mm::build_app_mem_space(elf_data).expect("build app mem space must succeed");

        let mut trap_context =
            unsafe { &*tcb.mem_space.trap_context_mut_ptr::<TrapContext>() }.clone();
        trap_context.set_user_sp(user_sp);
        trap_context.set_entry(entry);

        let trap_context_dest = mem_space.trap_context_mut_ptr();
        unsafe { *trap_context_dest = trap_context };

        tcb.mem_space = mem_space;
        tcb.name = path.to_string();

        Ok(())
    }
}

pub fn switch_task(current: *mut TaskContext, next: *const TaskContext) {
    extern "C" {
        fn _switch_task(current: *mut TaskContext, next: *const TaskContext);
    }

    unsafe { _switch_task(current, next) };
}

pub fn push_to_runq(tcb: TaskControlBlock) -> *mut TaskContext {
    TASK_MANAGER.lock().push_to_runq(tcb)
}

pub fn fetch_from_runq() -> Option<TaskControlBlock> {
    TASK_MANAGER.lock().fetch_from_runq()
}

pub fn create_tcb_by_app_name(name: &str) -> error::Result<TaskControlBlock> {
    TASK_MANAGER.lock().create_task(name)
}

pub fn fork_tcb(tcb: &TaskControlBlock) -> error::Result<TaskControlBlock> {
    TASK_MANAGER.lock().fork_task(tcb)
}

pub fn load_elf_in_task(path: &str, tcb: &mut TaskControlBlock) -> error::Result<()> {
    TASK_MANAGER.lock().load_elf_in_task(path, tcb)
}

pub fn list_apps() -> alloc::vec::Vec<alloc::string::String> {
    TASK_MANAGER.lock().app_loader.app_names()
}
