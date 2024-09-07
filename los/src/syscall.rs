mod fs;
mod proc;

use fs::sys_write;
use proc::{sys_exit, sys_task_yield};

use crate::{
    error::{self, Result},
    println,
};

#[derive(Debug)]
#[repr(usize)]
enum SyscallId {
    Write = 64,
    Exit = 93,
    Yield = 124,
}

impl TryFrom<usize> for SyscallId {
    type Error = error::KernelError;

    fn try_from(value: usize) -> Result<Self> {
        match value {
            64 => Ok(Self::Write),
            93 => Ok(Self::Exit),
            124 => Ok(Self::Yield),
            _ => Err(error::KernelError::InvalidSyscallId(value)),
        }
    }
}

pub fn syscall(id: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    match SyscallId::try_from(id) {
        Ok(id) => match id {
            SyscallId::Write => sys_write(arg0, arg1 as *const u8, arg2) as usize,
            SyscallId::Exit => sys_exit(arg0 as i32),
            SyscallId::Yield => sys_task_yield(),
        },
        Err(err) => {
            println!("[SYSCALL] parse syscall id failed: {:?}", err);
            -1i8 as usize
        }
    }
}
