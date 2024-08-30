#![no_std]
#![no_main]

use core::arch::asm;

use user::{self, println};

#[no_mangle]
fn main() -> i32 {
    foo();

    0
}

fn foo() {
    bar()
}

// use addr2line to get function name and filename
// addr2line -e <ELF_FILE> -C ADDRESS [ADDRESSES...]
fn bar() {
    let mut fp: usize;
    let mut pc: usize;
    unsafe {
        asm!(
            "mv {fp}, fp",
            "auipc {pc}, 0",
            fp = out(reg) fp,
            pc = out(reg) pc,
        );
    }

    println!("pc:{:#x}", pc);

    while fp != 0 {
        let ra = unsafe { *((fp - 8) as *const u64) as usize };
        println!("ra:{:#x} fp:{:#x}", ra, fp);

        fp = unsafe { *((fp - 16) as *const u64) as usize };
    }
}
