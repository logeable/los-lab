use core::{arch::asm, ffi::CStr};

use crate::TimeVal;

pub const SYS_WRITE: usize = 64;
pub const SYS_EXIT: usize = 93;
pub const SYS_SCHED_YIELD: usize = 124;
pub const SYS_GETTIMEOFDAY: usize = 169;
pub const SYS_FORK: usize = 220;
pub const SYS_EXEC: usize = 221;
pub const SYS_WAITPID: usize = 260;

pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
    syscall_3(SYS_WRITE, fd, buf.as_ptr() as usize, buf.len())
}

pub fn sys_exit(exit_code: usize) -> ! {
    syscall_1(SYS_EXIT, exit_code);

    unreachable!()
}

pub fn sys_sched_yield() -> isize {
    syscall_0(SYS_SCHED_YIELD)
}

pub fn sys_gettimeofday(tp: *mut TimeVal, tzp: usize) -> isize {
    syscall_2(SYS_GETTIMEOFDAY, tp as usize, tzp)
}

pub fn sys_fork() -> isize {
    syscall_0(SYS_FORK)
}

pub fn sys_exec(path: &CStr) -> isize {
    syscall_1(SYS_EXEC, path.as_ptr() as usize)
}

pub fn sys_wait(pid: isize, exit_code: &mut i32) -> isize {
    syscall_2(SYS_WAITPID, pid as usize, exit_code as *mut i32 as usize)
}

#[allow(dead_code)]
fn syscall_0(id: usize) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            in("a7") id,
            lateout("a0") ret
        );
    }

    ret
}

#[allow(dead_code)]
fn syscall_1(id: usize, arg0: usize) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            in("a7") id,
            inlateout("a0") arg0 => ret,
        );
    }

    ret
}

#[allow(dead_code)]
fn syscall_2(id: usize, arg0: usize, arg1: usize) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            in("a7") id,
            inlateout("a0") arg0 => ret,
            in("a1") arg1,
        );
    }

    ret
}

fn syscall_3(id: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            in("a7") id,
            inlateout("a0") arg0 => ret,
            in("a1") arg1,
            in("a2") arg2,
        );
    }

    ret
}
