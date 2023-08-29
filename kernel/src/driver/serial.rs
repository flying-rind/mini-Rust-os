//! 格式化输出至串口

use core::fmt;
use core::fmt::Write;
use spin::{Mutex, Once};
use uart_16550::SerialPort;
use x86_64::instructions::interrupts;

/// 串口驱动全局变量
pub static SERIAL: Once<Mutex<SerialPort>> = Once::new();

/// 初始化串口驱动
pub fn init() {
    let mut serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init();
    SERIAL.call_once(|| Mutex::new(serial_port));
}

/// 从串口中读取一个字符
pub fn receive() -> u8 {
    let serial = SERIAL.get().unwrap();
    serial.lock().receive()
}

/// 辅助打印结构，主要实现Write trait
struct Printer;
impl Write for Printer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        interrupts::without_interrupts(|| {
            SERIAL
                .get()
                .and_then(|serial| Some(serial.lock().write_str(s)))
                .expect("Uninit serial")
                .unwrap();
        });
        Ok(())
    }
}

/// 打印至串口
pub fn print(args: fmt::Arguments) {
    // write_fmt函数在Write trait中定义，因此需要实现Write trait
    Printer.write_fmt(args).unwrap();
}

/// 格式化打印至串口（无换行）
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::driver::serial::print(format_args!($($arg)*))
    };
}

/// 格式化打印至串口（有换行）
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
