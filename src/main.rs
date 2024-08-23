#![no_std]
#![no_main]
use core::{arch::global_asm, include_str, panic::PanicInfo};

global_asm!(include_str!("entry.asm"));

#[no_mangle]
fn rust_main() {
    extern "C" {
        fn skernel();
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbtstack();
        fn ebtstack();
        fn sbss();
        fn ebss();
        fn ekernel();
    }

    clear_bss();

    loop {}
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
fn panic_handler(_panic_info: &PanicInfo) -> ! {
    loop {}
}
