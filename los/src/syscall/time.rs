use core::mem;

use crate::{
    println, task,
    timer::{self, TimeVal},
};

pub fn sys_gettimeofday(tp: *mut TimeVal, _tzp: usize) -> isize {
    match task::translate_by_current_task_pagetable(tp as usize, mem::size_of::<TimeVal>()) {
        Ok(chunks) => {
            let chunk = chunks.first().unwrap();
            let tp = unsafe { &mut *(chunk.as_ptr() as *mut TimeVal) };
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
