#![no_std]
#![no_main]

use los_user::{self, println};

#[no_mangle]
fn main() -> i32 {
    let mut sum = 0;
    while sum < i32::MAX {
        sum += 1;
    }
    0
}
