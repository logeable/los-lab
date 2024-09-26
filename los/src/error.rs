use core::fmt::Debug;

use alloc::string::String;

#[derive(Debug)]
#[allow(dead_code)]
pub enum KernelError {
    InvalidSyscallId(String),
    AllocFrame(String),
    CreateMemorySpace(String),
    PagetableMap(String),
    MapArea(String),
    VpnTranslate(String),
}

impl core::error::Error for KernelError {}

impl core::fmt::Display for KernelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Result<T> = core::result::Result<T, KernelError>;
