use crate::{println, task};

pub fn sys_exit(exit_code: i32) -> ! {
    {
        println!("app exit_code: {}", exit_code,);
    }

    task::exit_current_task_and_schedule();

    unreachable!()
}

pub fn sys_task_yield() -> usize {
    task::suspend_current_task_and_schedule();

    0
}
