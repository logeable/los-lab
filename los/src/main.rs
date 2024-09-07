#![no_std]
#![no_main]
mod console;
mod error;
mod sbi;
mod syscall;
mod task;
mod trap;

use core::{arch::global_asm, panic::PanicInfo};

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("trap.asm"));
global_asm!(include_str!("app.asm"));

#[no_mangle]
fn rust_main() {
    clear_bss();
    trap::init();
    task::init();

    print_kernel_info();

    task::schedule();
}

fn print_kernel_info() {
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

    fn color_print(name: &str, start: usize, end: usize) {
        println!("{:10}: [{:#x}..{:#x}]", name, start, end,);
    }

    color_print("kernel", skernel as usize, ekernel as usize);
    color_print(".text", stext as usize, etext as usize);
    color_print(".rodata", srodata as usize, erodata as usize);
    color_print(".data", sdata as usize, edata as usize);
    color_print(".btstack", sbtstack as usize, ebtstack as usize);
    color_print(".bss", sbss as usize, ebss as usize);
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
    println!("{}", panic_info);

    sbi::shutdown(true);
}
