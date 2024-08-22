#![no_std]
#![no_main]
use core::{arch::global_asm, include_str, panic::PanicInfo};

global_asm!(include_str!("entry.asm"));

#[panic_handler]
fn panic_handler(_panic_info: &PanicInfo) -> ! {
    loop {}
}
