#![no_std]
#![no_main]

use core::arch::asm;
use user_rt::{self, entry};

entry!(main);

fn main() -> i32 {
    unsafe {
        asm!("csrr t0, sstatus");
    }

    0
}
