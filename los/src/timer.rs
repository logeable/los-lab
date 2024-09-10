use riscv::register::time;

use crate::sbi;

const CLOCK_FREQ: usize = 10000000;
const TICKS_PER_SEC: usize = 100;
const US_PER_SEC: usize = 1000000;

pub fn init() {
    set_next_trigger()
}

pub fn get_time() -> TimeVal {
    let ticks = time::read();

    let usec = (ticks / (CLOCK_FREQ / US_PER_SEC)) as u64;

    TimeVal { sec: 0, usec }
}

pub fn set_next_trigger() {
    sbi::set_timer(time::read() + CLOCK_FREQ / TICKS_PER_SEC);
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TimeVal {
    pub sec: u64,
    pub usec: u64,
}
