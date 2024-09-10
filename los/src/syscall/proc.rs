use crate::{println, task};

pub fn sys_exit(exit_code: i32) -> ! {
    println!(
        "app {:?} exit, code: {}",
        task::get_current_task_name(),
        exit_code,
    );

    task::exit_current_task_and_schedule()
}

pub fn sys_sched_yield() -> isize {
    task::suspend_current_task_and_schedule();

    0
}
