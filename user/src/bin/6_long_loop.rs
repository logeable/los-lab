#![no_std]
#![no_main]

use user::{entry, println, sched_yield};

entry!(main);

fn main() -> i32 {
    let mut sum = 0;
    while sum < i16::MAX {
        sum += 1;
        println!("{}", sum);
        sched_yield();
    }
    0
}
