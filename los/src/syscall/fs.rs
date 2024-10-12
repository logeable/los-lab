use alloc::vec;

use crate::{mm, println, sbi, task::processor};

const STDIN: usize = 0;
const STDOUT: usize = 1;

const READ_BUF_SIZE: usize = 1 << 10;

pub fn sys_read(fd: usize, user_buf: *mut u8, len: usize) -> isize {
    match fd {
        STDIN => {
            let mut read_buf = vec![0u8; READ_BUF_SIZE.min(len)];
            let read_len = sbi::console_read_bytes(read_buf.as_mut_slice());

            assert!(read_len as usize <= len);

            if read_len <= 0 {
                return read_len;
            }

            let satp = processor::get_current_task_satp();
            match mm::PageTable::from_satp(satp).translate_bytes((user_buf as usize).into(), len) {
                Ok(chunks) => {
                    let mut read_data = &read_buf[..read_len as usize];

                    for chunk in chunks {
                        let chunk_len = chunk.len();
                        let read_len = read_data.len();
                        let len = chunk_len.min(read_len);

                        chunk[..len].copy_from_slice(&read_data[..len]);
                        read_data = &read_data[len..];
                    }
                }
                Err(err) => {
                    println!("[FS] translate failed: {:?}", err);
                    return -1;
                }
            }

            read_len
        }
        _ => {
            panic!("[FS] read from invalid fd: {}", fd);
        }
    }
}

pub fn sys_write(fd: usize, data: *const u8, len: usize) -> isize {
    match fd {
        STDOUT => {
            let satp = processor::get_current_task_satp();
            match mm::PageTable::from_satp(satp).translate_bytes((data as usize).into(), len) {
                Ok(chunks) => {
                    let mut len = 0usize;
                    for chunk in chunks.iter() {
                        for b in chunk.iter() {
                            sbi::console_write_byte(*b as usize);
                        }
                        len += chunk.len();
                    }

                    len as isize
                }
                Err(err) => {
                    println!("[FS] translate failed: {:?}", err);
                    return -1;
                }
            }
        }
        _ => {
            panic!("[FS] write to inavlid fd: {}", fd);
        }
    }
}
