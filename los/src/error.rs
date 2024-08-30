use core::fmt::Debug;

#[derive(Debug)]
#[allow(dead_code)]
pub enum KernelError {
    InvalidSyscallId(usize),
}

impl core::error::Error for KernelError {}

impl core::fmt::Display for KernelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
