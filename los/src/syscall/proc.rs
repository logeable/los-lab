use crate::{
    mm, println,
    task::processor::{self, WaitChildArg},
};

pub fn sys_exit(exit_code: i32) -> ! {
    processor::exit_current_task_and_schedule(exit_code)
}

pub fn sys_sched_yield() -> isize {
    processor::suspend_current_task_and_schedule();

    0
}

pub fn sys_fork() -> isize {
    match processor::fork_current_task() {
        Ok(pid) => pid as isize,
        Err(err) => {
            println!("[PROC] sys fork failed: {:?}", err);
            return -1;
        }
    }
}

pub fn sys_exec(path: *const u8) -> isize {
    let satp = processor::get_current_task_satp();
    let page_table = mm::PageTable::from_satp(satp);
    match page_table.translate_c_str((path as usize).into()) {
        Ok(path) => match processor::exec_in_tcb(&path) {
            Ok(()) => 0,
            Err(err) => {
                println!("[PROC] sys exec failed: {:?}", err);
                -1
            }
        },
        Err(err) => {
            println!("[PROC] translate path failed: {:?}", err);
            -2
        }
    }
}

pub fn sys_wait(pid: isize, exit_code: *mut i32) -> isize {
    let wait_child_arg = match WaitChildArg::from_pid(pid) {
        Ok(wait_child_arg) => wait_child_arg,
        Err(err) => {
            println!("[PROC] build WaitChildArg failed: {:?}", err);
            return -1;
        }
    };

    match processor::wait_child_exit(wait_child_arg).expect("wait child must succeed") {
        Some(result) => {
            let satp = processor::get_current_task_satp();
            match mm::PageTable::from_satp(satp).translate_write(exit_code, &result.exit_code) {
                Ok(_) => result.pid as isize,
                Err(err) => {
                    println!("[PROC] write exit_code to user buf failed: {:?}", err);
                    -2
                }
            }
        }
        None => 0,
    }
}
