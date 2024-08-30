#![no_std]

pub mod console;
mod syscall;

use core::panic::PanicInfo;

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
    println!("APP PANIC: {:?}", panic_info);
    loop {}
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

fn write(fd: usize, buf: &[u8]) -> isize {
    syscall::sys_write(fd, buf)
}
fn exit(exit_code: i32) -> ! {
    syscall::sys_exit(exit_code as usize)
}
