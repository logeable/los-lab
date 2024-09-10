use crate::timer::{self, TimeVal};

pub fn sys_gettimeofday(tp: *mut TimeVal, _tzp: usize) -> isize {
    let tp = unsafe { &mut *tp };
    let t = timer::get_time();

    tp.sec = t.sec;
    tp.usec = t.usec;

    0
}
