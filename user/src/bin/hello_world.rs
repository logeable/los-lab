#![no_std]
#![no_main]

use user::{self, entry, println};

entry!(main);

fn main() -> i32 {
    println!("hello world!");

    0
}
