#![no_std]
#![no_main]

use core::arch::asm;

use los_user;

#[no_mangle]
fn main() -> i32 {
    unsafe {
        asm!("csrr t0, sstatus");
    }

    0
}
