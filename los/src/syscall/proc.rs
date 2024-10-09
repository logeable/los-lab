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

pub fn sys_fork() -> isize {
    unimplemented!()
}

pub fn sys_exec(path: *const u8) -> isize {
    unimplemented!()
}

pub fn sys_wait(pid: isize, exit_code: *mut i32) -> isize {
    unimplemented!()
}
