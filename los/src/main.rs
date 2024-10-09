#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

mod config;
mod console;
mod device_tree;
mod error;
mod mm;
mod sbi;
mod syscall;
mod task;
mod timer;
mod trap;

use core::{arch::global_asm, panic::PanicInfo};

use device_tree::{get_device_info, DeviceInfo};

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("trap.asm"));
global_asm!(include_str!("app.asm"));

#[no_mangle]
extern "C" fn rust_main(_hartid: usize, device_tree_pa: usize) -> ! {
    #[cfg(test)]
    {
        test_main();
        sbi::shutdown(false);
    }

    clear_bss();
    device_tree::init(device_tree_pa);

    let device_info = get_device_info();
    print_kernel_info(&device_info);

    mm::init(&device_info);
    trap::init();
    timer::init(&device_info);

    task::run_tasks();
}

fn print_kernel_info(device_info: &DeviceInfo) {
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

    color_print("memory", device_info.memory.start, device_info.memory.end);
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

pub trait TestCase {
    fn run(&self);
}

impl<T> TestCase for T
where
    T: Fn(),
{
    fn run(&self) {
        print!("{}...\t", core::any::type_name::<T>());
        self();
    }
}

#[cfg(test)]
fn test_runner(test_cases: &[&dyn TestCase]) {
    println!("Running {} tests", test_cases.len());
    for case in test_cases {
        case.run();
        println!("[ok]");
    }
    println!("test finished");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_working() {
        assert!(true, "test working")
    }
}
