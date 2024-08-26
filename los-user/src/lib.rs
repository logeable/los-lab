#![no_std]
#![no_main]
#![feature(linkage)]

pub mod console;
mod syscall;

use core::panic::PanicInfo;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
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

#[no_mangle]
#[linkage = "weak"]
fn main() -> i32 {
    unimplemented!()
}

#[panic_handler]
fn panic_handler(_panic_info: &PanicInfo) -> ! {
    loop {}
}

fn write(fd: usize, buf: &[u8]) -> isize {
    syscall::sys_write(fd, buf)
}
fn exit(exit_code: i32) -> ! {
    syscall::sys_exit(exit_code as usize)
}
