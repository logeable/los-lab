#![no_std]
#![no_main]

use user::{self, entry, gettimeofday, println, sched_yield};

entry!(main);

fn main() -> i32 {
    let t = gettimeofday().unwrap();
    let v = t.usec;
    println!("start: {:?}", t);

    loop {
        let t = gettimeofday().unwrap();
        if t.usec > v + 10000000 {
            println!("done: {:?}", t);
            return 0;
        }

        sched_yield();
    }
}
