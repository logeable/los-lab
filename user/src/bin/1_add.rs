#![no_std]
#![no_main]

use user::{self, println};

#[no_mangle]
fn main() -> i32 {
    let a = 1;
    let b = 2;
    println!("{} + {} = {}", a, b, add(a, b));

    0
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
