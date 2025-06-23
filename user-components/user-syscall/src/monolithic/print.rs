use super::write;
use crate::monolithic::read;
use core::fmt::{self, Write};

// const STDIN: usize = 0;
const STDOUT: usize = 1;
const STDIN: usize = 0;

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let _ = write(STDOUT, s.as_bytes());
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[cfg(feature = "monolithic")]
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        user_syscall::monolithic::print::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[cfg(feature = "monolithic")]
#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        user_syscall::monolithic::print::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

pub fn getchar() -> u8 {
    let mut c = [0u8; 1];
    let _ = read(STDIN, &mut c);
    c[0]
}
