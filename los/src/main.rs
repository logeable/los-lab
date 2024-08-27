#![no_std]
#![no_main]
mod batch;
mod console;
mod error;
mod sbi;
mod syscall;
mod trap;

use ansi_rgb::cyan_blue;
use ansi_rgb::{red, Foreground};
use batch::APP_LOADER;
use core::{arch::global_asm, panic::PanicInfo};

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!(
    "../../target/riscv64gc-unknown-none-elf/release/app.asm"
));

#[no_mangle]
fn rust_main() {
    clear_bss();
    print_kernel_info();

    trap::init();

    {
        let app_loader = APP_LOADER.lock();
        println!("app numbers: {}", app_loader.app_number());
        for i in 0..app_loader.app_number() {
            println!("loading app {}: {:?}", i, app_loader.app_info(i as usize));
        }
    }

    batch::run_next_app();
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
        println!(
            "{}",
            format_args!("{:10}: [{:#x}..{:#x}]", name, start, end,).fg(cyan_blue())
        );
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
    println!("{}", panic_info.fg(red()));

    sbi::shutdown(true);
}
