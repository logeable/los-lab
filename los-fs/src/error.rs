use alloc::string::String;

#[derive(Debug)]
pub enum Error {
    ReadBlock(String),
    WriteBlock(String),
    NoFreeCache,
}

pub type Result<T> = core::result::Result<T, Error>;
