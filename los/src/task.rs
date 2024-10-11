mod initproc;
mod loader;
mod manager;
mod pid;
mod processor;
mod tcb;

use crate::println;

pub use pid::Pid;
pub use processor::exec_in_tcb;
pub use processor::exit_current_task_and_schedule;
pub use processor::fork_current_task;
pub use processor::get_current_task_name;
pub use processor::get_current_task_satp;
pub use processor::get_current_task_trap_context;
pub use processor::run_tasks;
pub use processor::suspend_current_task_and_schedule;

pub fn init() {
    initproc::init();
}

pub(crate) fn print_apps() {
    for (i, name) in manager::list_apps().iter().enumerate() {
        println!("{}: {}", i, name);
    }
}
