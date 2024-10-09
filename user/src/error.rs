#[derive(Debug)]
pub enum Error {
    SyscallError(isize),
    CastToCStr,
    PathTooLong,
    UnexpectedEof,
}

// impl core::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Error: {:?}", self)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
