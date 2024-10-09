use crate::{println, sbi, task};

const STDOUT: usize = 1;

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    unimplemented!()
}

pub fn sys_write(fd: usize, data: *const u8, len: usize) -> isize {
    match fd {
        STDOUT => match task::translate_by_current_task_pagetable(data as usize, len) {
            Ok(chunks) => {
                let mut len = 0usize;
                for chunk in chunks.iter() {
                    for b in chunk.iter() {
                        sbi::console_putchar(*b as usize);
                    }
                    len += chunk.len();
                }

                len as isize
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
