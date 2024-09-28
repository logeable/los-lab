use crate::{println, sbi, task};
use core::slice;

const STDOUT: usize = 1;

pub fn sys_write(fd: usize, data: *const u8, len: usize) -> isize {
    match fd {
        STDOUT => match task::translate_by_current_task_pagetable(data as usize) {
            Ok(pa) => {
                let buf = unsafe { slice::from_raw_parts(pa as *const u8, len) };
                for b in buf {
                    sbi::console_putchar(*b as usize);
                }
                buf.len() as isize
            }
            Err(err) => {
                println!("translate failed: {:?}", err);
                return -1;
            }
        },
        _ => {
            panic!("write to inavlid fd: {}", fd);
        }
    }
}
