use crate::{
    println, task,
    timer::{self, TimeVal},
};

pub fn sys_gettimeofday(tp: *mut TimeVal, _tzp: usize) -> isize {
    match task::translate_by_current_task_pagetable(tp as usize) {
        Ok(pa) => {
            let tp = unsafe { &mut *(pa as *mut TimeVal) };
            let t = timer::get_time();

            tp.sec = t.sec;
            tp.usec = t.usec;

            0
        }
        Err(err) => {
            println!("translate failed: {:?}", err);
            return -1;
        }
    }
}
