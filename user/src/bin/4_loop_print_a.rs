#![no_std]
#![no_main]

use user::{self, entry, println, sched_yield};

entry!(main);

fn main() -> i32 {
    for i in 0..10 {
        println!("A: {}", i);
        sched_yield();
    }

    0
}
