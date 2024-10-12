use core::mem;

use crate::{
    mm, println,
    task::processor,
    timer::{self, TimeVal},
};

pub fn sys_gettimeofday(tp: *mut TimeVal, _tzp: usize) -> isize {
    let satp = processor::get_current_task_satp();
    match mm::PageTable::from_satp(satp)
        .translate_bytes((tp as usize).into(), mem::size_of::<TimeVal>())
    {
        Ok(chunks) => {
            let chunk = chunks.first().unwrap();
            let tp = unsafe { &mut *(chunk.as_ptr() as *mut TimeVal) };
            let t = timer::get_time();

            tp.sec = t.sec;
            tp.usec = t.usec;

            0
        }
        Err(err) => {
            println!("[TIME] translate failed: {:?}", err);
            return -1;
        }
    }
}
