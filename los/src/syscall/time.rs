use crate::timer;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TimeVal {
    pub sec: u64,
    pub usec: u64,
}

pub fn sys_gettimeofday(tp: *mut TimeVal, _tzp: usize) -> isize {
    let tp = unsafe { &mut *tp };
    tp.usec = timer::get_time() as u64;

    0
}
