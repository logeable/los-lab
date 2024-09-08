use crate::{println, task::loader::AppLoader, trap::TrapContext};

use super::{TaskContext, TaskControlBlock, TaskStatus};
use core::{arch::global_asm, mem};
use lazy_static::lazy_static;
use spin::Mutex;

global_asm!(include_str!("switch.asm"));

const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub(super) const MAX_APPS: usize = 100;

static KERNEL_STACKS: [KernelStack; MAX_APPS] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APPS];

static USER_STACKS: [UserStack; MAX_APPS] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_APPS];

lazy_static! {
    pub static ref TASK_MANAGER: Mutex<TaskManager> = {
        let manager = TaskManager::new();

        Mutex::new(manager)
    };
}

pub struct TaskManager {
    tasks: [Option<TaskControlBlock>; MAX_APPS],
    current_task_index: Option<usize>,
}

impl TaskManager {
    fn new() -> Self {
        extern "C" {
            fn _s_trap_return() -> !;
        }

        let app_loader = AppLoader::new();
        let number_of_task = app_loader.get_number_of_app();
        let mut tasks = [None; MAX_APPS];
        for i in 0..number_of_task {
            let app = app_loader.get_app_info(i).expect("app index should valid");

            let user_sp = USER_STACKS[i].get_sp();
            let trap_context =
                KERNEL_STACKS[i].push_trap_context(TrapContext::init(app.entry, user_sp));
            let kernel_sp = trap_context as usize;

            tasks[i] = Some(TaskControlBlock::init(
                app.name,
                _s_trap_return as usize,
                kernel_sp,
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
            if let Some(task) = &mut self.tasks[idx] {
                if task.status == TaskStatus::Ready {
                    found_idx = Some(idx);
                    break;
                }
            }
        }

        found_idx.map(|idx| (idx, self.tasks[idx].as_mut().unwrap()))
    }

    fn get_current_mut(&mut self) -> Option<&mut TaskControlBlock> {
        match self.current_task_index {
            Some(idx) => self.tasks[idx].as_mut(),
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
    //loop {}
}

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    fn push_trap_context(&self, ctx: TrapContext) -> *mut TrapContext {
        let sp = self.get_sp() - mem::size_of::<TrapContext>();
        let ctx_ptr = sp as *mut TrapContext;
        unsafe { ctx_ptr.write(ctx) };
        ctx_ptr
    }
}

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub fn exit_current_task_and_schedule() {
    TASK_MANAGER.lock().get_current_mut().unwrap().status = TaskStatus::Exited;
    schedule();
}

pub fn suspend_current_task_and_schedule() {
    TASK_MANAGER.lock().get_current_mut().unwrap().status = TaskStatus::Ready;
    schedule();
}
