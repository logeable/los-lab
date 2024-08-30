#![no_std]
#![no_main]

use user_rt::{entry, println};

entry!(main);

fn main() -> i32 {
    println!("hello world!");

    0
}
