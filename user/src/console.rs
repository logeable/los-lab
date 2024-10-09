use core::fmt::Write;

use crate::{read, write};

pub const STDOUT: usize = 1;
pub const STDIN: usize = 0;

pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write(STDOUT, s.as_bytes());
        Ok(())
    }
}

pub fn print(args: core::fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
	($fmt: literal $(, $($arg: tt)+)?) => {
		$crate::console::print(format_args!($fmt $(, $($arg)+)?));
	}
}

#[macro_export]
macro_rules! println {
	($fmt: literal $(, $($arg: tt)+)?) => {
		$crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
	}
}

pub struct Stdin;

impl Stdin {
    pub fn read(buf: &mut [u8]) -> crate::error::Result<usize> {
        read(STDIN, buf)
    }

    pub fn read_u8() -> crate::error::Result<u8> {
        let mut buf = [0u8; 1];

        let len = Self::read(&mut buf)?;

        if len == 0 {
            return Err(crate::error::Error::UnexpectedEof);
        }

        Ok(buf[0])
    }
}
