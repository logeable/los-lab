use core::ffi::{c_char, CStr};

use crate::{mm, println, task};

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
    match task::fork_current_task() {
        Ok(pid) => pid as isize,
        Err(err) => {
            println!("[PROC] sys fork failed: {:?}", err);
            return -1;
        }
    }
}

pub fn sys_exec(path: *const u8) -> isize {
    let satp = task::get_current_task_satp();
    let page_table = mm::PageTable::from_satp(satp);
    match page_table.translate_c_str((path as usize).into()) {
        Ok(path) => match task::exec_in_tcb(&path) {
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
    0
}
