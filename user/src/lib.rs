#![no_std]
#![no_main]

pub mod console;
mod error;
mod syscall;

use core::{ffi::CStr, panic::PanicInfo};
use error::{Error, Result};

const MAX_PATH_LEN: usize = 128;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    extern "C" {
        fn __main() -> i32;
    }

    clear_bss();
    exit(unsafe { __main() });
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    (sbss as usize..ebss as usize).for_each(|addr| {
        unsafe { (addr as *mut u8).write_volatile(0) };
    });
}

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    println!("APP PANIC: {}", panic_info);

    exit(1)
}

#[macro_export]
macro_rules! entry {
    ($path:path) => {
        #[export_name = "__main"]
        pub unsafe fn __main() -> i32 {
            let f: fn() -> i32 = $path;

            f()
        }
    };
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    syscall::sys_write(fd, buf)
}

pub fn exit(exit_code: i32) -> ! {
    syscall::sys_exit(exit_code as usize)
}

pub fn sched_yield() -> isize {
    syscall::sys_sched_yield()
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TimeVal {
    pub sec: u64,
    pub usec: u64,
}

pub fn gettimeofday() -> Result<TimeVal> {
    let mut t = TimeVal { sec: 0, usec: 0 };

    let ret = syscall::sys_gettimeofday(&mut t, 0);
    if ret != 0 {
        return Err(Error::SyscallError(ret));
    }

    Ok(t)
}

pub fn fork() -> Result<usize> {
    let ret = syscall::sys_fork();
    if ret < 0 {
        return Err(Error::SyscallError(ret));
    }

    Ok(ret as usize)
}

pub fn exec(path: &str) -> Result<()> {
    if path.len() + 1 > MAX_PATH_LEN {
        return Err(Error::PathTooLong);
    }

    let bytes = path.as_bytes();
    let mut buf = [0u8; MAX_PATH_LEN];
    buf.fill(0);
    buf[..bytes.len()].copy_from_slice(bytes);

    let cstr = CStr::from_bytes_until_nul(&buf).map_err(|_| Error::CastToCStr)?;

    let ret = syscall::sys_exec(cstr);
    if ret < 0 {
        return Err(Error::SyscallError(ret));
    }

    Ok(())
}

pub fn wait() -> Result<usize> {
    let mut exit_code = 0;

    loop {
        let ret = syscall::sys_wait(-1, &mut exit_code);
        if ret < 0 {
            return Err(Error::SyscallError(ret));
        } else if ret == 0 {
            sched_yield();
        } else {
            return Ok(ret as usize);
        }
    }
}

pub fn waitpid(pid: usize) -> Result<usize> {
    let mut exit_code = 0;

    loop {
        let ret = syscall::sys_wait(pid as isize, &mut exit_code);
        if ret < 0 {
            return Err(Error::SyscallError(ret));
        } else if ret == 0 {
            sched_yield();
        } else {
            return Ok(ret as usize);
        }
    }
}
