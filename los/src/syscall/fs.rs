use crate::sbi;
use core::slice;

const STDOUT: usize = 1;

pub fn sys_write(fd: usize, data: *const u8, len: usize) -> isize {
    match fd {
        STDOUT => {
            let buf = unsafe { slice::from_raw_parts(data, len) };
            for b in buf {
                sbi::console_putchar(*b as usize);
                // print!("{:x} ", b);
            }
            buf.len() as isize
        }
        _ => {
            panic!("write to inavlid fd: {}", fd);
        }
    }
}
