#![no_std]
#![no_main]

use los_user::{self, println};

#[no_mangle]
fn main() -> i32 {
    println!("hello world!");

    0
}
