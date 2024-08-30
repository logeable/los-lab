#![no_std]
#![no_main]

use user::entry;

entry!(main);

fn main() -> i32 {
    let mut sum = 0;
    while sum < i32::MAX {
        sum += 1;
    }
    0
}
