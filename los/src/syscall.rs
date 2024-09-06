mod fs;
mod proc;

use fs::sys_write;
use proc::sys_exit;

use crate::error::{self, Result};

#[repr(usize)]
enum SyscallId {
    Write = 64,
    Exit = 93,
}

impl TryFrom<usize> for SyscallId {
    type Error = error::KernelError;

    fn try_from(value: usize) -> Result<Self> {
        match value {
            64 => Ok(Self::Write),
            93 => Ok(Self::Exit),
            _ => Err(error::KernelError::InvalidSyscallId(value)),
        }
    }
}

pub fn syscall(id: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    match SyscallId::try_from(id) {
        Ok(id) => match id {
            SyscallId::Write => sys_write(arg0, arg1 as *const u8, arg2) as usize,
            SyscallId::Exit => sys_exit(arg0 as i32),
        },
        Err(_) => -1i8 as usize,
    }
}
