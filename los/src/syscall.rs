mod fs;
mod proc;
mod time;

use crate::{println, timer::TimeVal};
use fs::{sys_read, sys_write};
use proc::{sys_exec, sys_exit, sys_fork, sys_sched_yield, sys_wait};
use time::sys_gettimeofday;

pub const SYS_READ: usize = 63;
pub const SYS_WRITE: usize = 64;
pub const SYS_EXIT: usize = 93;
pub const SYS_SCHED_YIELD: usize = 124;
pub const SYS_GETTIMEOFDAY: usize = 169;
pub const SYS_FORK: usize = 220;
pub const SYS_EXEC: usize = 221;
pub const SYS_WAITPID: usize = 260;
pub fn syscall(id: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    match id {
        SYS_READ => sys_read(arg0, arg1 as *mut u8, arg2) as usize,
        SYS_WRITE => sys_write(arg0, arg1 as *const u8, arg2) as usize,
        SYS_EXIT => sys_exit(arg0 as i32),
        SYS_SCHED_YIELD => sys_sched_yield() as usize,
        SYS_GETTIMEOFDAY => sys_gettimeofday(arg0 as *mut TimeVal, arg1) as usize,
        SYS_FORK => sys_fork() as usize,
        SYS_EXEC => sys_exec(arg0 as *const u8) as usize,
        SYS_WAITPID => sys_wait(arg0 as isize, arg1 as *mut i32) as usize,
        _ => {
            println!("[SYSCALL] parse syscall id failed: {}", id);
            -1i8 as usize
        }
    }
}
