use riscv::register::time;

use crate::sbi;

const CLOCK_FREQ: usize = 10000000;
const TICKS_PER_SEC: usize = 100;

pub fn init() {
    set_next_trigger()
}

pub fn get_time() -> usize {
    time::read()
}

pub fn set_next_trigger() {
    sbi::set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
