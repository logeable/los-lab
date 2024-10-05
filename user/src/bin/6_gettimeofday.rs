#![no_std]
#![no_main]

use user::{self, entry, gettimeofday, println, sched_yield};

entry!(main);

fn main() -> i32 {
    let start = gettimeofday().unwrap();
    println!("start: {:?}", start);

    loop {
        let t = gettimeofday().unwrap();
        if t.sec - start.sec >= 3 {
            println!("done: {:?}", t);
            return 0;
        }

        sched_yield();
    }
}
