use super::write;

use core::fmt::{self, Write};

const STDIN: usize = 0;
const STDOUT: usize = 1;

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write(STDOUT, s.as_bytes());
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
