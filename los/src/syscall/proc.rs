use crate::{println, task};

pub fn sys_exit(exit_code: i32) -> ! {
    {
        println!("app exit_code: {}", exit_code,);
    }

    task::exit_current_task_and_schedule();

    unreachable!()
}

pub fn sys_task_yield() -> usize {
    println!("before {:?}", task::get_currrent_tcb());
    task::suspend_current_task_and_schedule();
    println!("after {:?}", task::get_currrent_tcb());

    0
}
