#![no_std]
#![no_main]

use user::{self, entry, println, sched_yield};

entry!(main);

fn main() -> i32 {
    for i in 0..20 {
        println!("B: {}", i);
        sched_yield();
    }

    0
}
