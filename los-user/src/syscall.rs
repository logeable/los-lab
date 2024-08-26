use core::arch::asm;

const SYS_WRITE: usize = 64;
const SYS_EXIT: usize = 93;

pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
    syscall_3(SYS_WRITE, fd, buf.as_ptr() as usize, buf.len())
}

pub fn sys_exit(exit_code: usize) -> ! {
    syscall_1(SYS_EXIT, exit_code);

    unreachable!()
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
