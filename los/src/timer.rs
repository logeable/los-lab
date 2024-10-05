use riscv::register::time;

use crate::{device_tree::DeviceInfo, sbi};

static mut TICKS_PER_SEC: usize = 0;

const MS_PER_TIME_SLICE: usize = 10;
const MS_PER_SEC: usize = 1000;
const US_PER_SEC: usize = MS_PER_SEC * 1000;

pub fn init(device_info: &DeviceInfo) {
    unsafe { TICKS_PER_SEC = device_info.cpu_time_base_freq };
    set_next_trigger()
}

pub fn get_time() -> TimeVal {
    let usec = (time::read() / (get_ticks_per_sec() / US_PER_SEC)) as u64;
    let sec = usec / US_PER_SEC as u64;
    let usec = usec % US_PER_SEC as u64;
    TimeVal { sec, usec }
}

pub fn set_next_trigger() {
    sbi::set_timer(time::read() + get_ticks_per_sec() / MS_PER_SEC * MS_PER_TIME_SLICE);
}

fn get_ticks_per_sec() -> usize {
    unsafe { TICKS_PER_SEC }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TimeVal {
    pub sec: u64,
    pub usec: u64,
}
