//! 串口相关接口

use bitflags::bitflags;
use core::fmt::{self, Write};
use spin::{Mutex, Once};
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

// uart16550
static SERIAL: Once<Mutex<SerialPort>> = Once::new();

/// 串口初始化
pub fn init(base_addr: usize) {
    let serial = Mutex::new(unsafe { SerialPort::new(base_addr as u16) });
    serial.lock().init();
    SERIAL.call_once(move || serial);
}

/// 格式化输出
pub fn serial_print(args: core::fmt::Arguments) {
    if let Some(ref mut serial) = SERIAL.get() {
        serial
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    } else {
        panic!("Printing to an uninitualised serial");
    }
}

/// 发送字节
pub fn serial_send(data: u8) {
    if let Some(ref mut serial) = SERIAL.get() {
        serial.lock().send(data);
    } else {
        panic!("Printing to an uninitualised serial");
    };
}

/// 接收字节，阻塞式地
pub fn serial_receive() -> u8 {
    if let Some(ref mut serial) = SERIAL.get() {
        serial.lock().receive()
    } else {
        panic!("Printing to an uninitualised serial");
    }
}

/// 清除中断
pub fn clear_irq() {}

/// 打开中断
pub fn enable_irq() {}

/// 关闭中断
pub fn disable_irq() {}

macro_rules! wait_for {
    ($cond:expr) => {
        while !$cond {
            core::hint::spin_loop()
        }
    };
}

bitflags! {
    /// Line status flags
    struct LineStsFlags: u8 {
        const INPUT_FULL = 1;
        // 1 to 4 unknown
        const OUTPUT_EMPTY = 1 << 5;
        // 6 and 7 unknown
    }
}
struct SerialPort {
    data: Port<u8>,
    int_en: PortWriteOnly<u8>,
    fifo_ctrl: PortWriteOnly<u8>,
    line_ctrl: PortWriteOnly<u8>,
    modem_ctrl: PortWriteOnly<u8>,
    line_sts: PortReadOnly<u8>,
}

#[allow(dead_code)]
impl SerialPort {
    /// Creates a new serial port interface on the given I/O port.
    ///
    /// This function is unsafe because the caller must ensure that the given base address
    /// really points to a serial port device.
    unsafe fn new(base: u16) -> Self {
        Self {
            data: Port::new(base),
            int_en: PortWriteOnly::new(base + 1),
            fifo_ctrl: PortWriteOnly::new(base + 2),
            line_ctrl: PortWriteOnly::new(base + 3),
            modem_ctrl: PortWriteOnly::new(base + 4),
            line_sts: PortReadOnly::new(base + 5),
        }
    }

    /// Initializes the serial port.
    ///
    /// The default configuration of [38400/8-N-1](https://en.wikipedia.org/wiki/8-N-1) is used.
    fn init(&mut self) {
        unsafe {
            // Disable interrupts
            self.int_en.write(0x00);

            // Enable DLAB
            self.line_ctrl.write(0x80);

            // Set maximum speed to 38400 bps by configuring DLL and DLM
            self.data.write(0x03);
            self.int_en.write(0x00);

            // Disable DLAB and set data word length to 8 bits
            self.line_ctrl.write(0x03);

            // Enable FIFO, clear TX/RX queues and
            // set interrupt watermark at 14 bytes
            self.fifo_ctrl.write(0xC7);

            // Mark data terminal ready, signal request to send
            // and enable auxilliary output #2 (used as interrupt line for CPU)
            self.modem_ctrl.write(0x0B);

            // Enable interrupts
            self.int_en.write(0x01);
        }
    }

    fn line_sts(&mut self) -> LineStsFlags {
        unsafe { LineStsFlags::from_bits_truncate(self.line_sts.read()) }
    }

    /// Sends a byte on the serial port.
    fn send(&mut self, data: u8) {
        unsafe {
            match data {
                8 | 0x7F => {
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self.data.write(8);
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self.data.write(b' ');
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self.data.write(8)
                }
                _ => {
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self.data.write(data);
                }
            }
        }
    }

    /// Sends a raw byte on the serial port, intended for binary data.
    fn send_raw(&mut self, data: u8) {
        unsafe {
            wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
            self.data.write(data);
        }
    }

    /// Receives a byte on the serial port.
    fn receive(&mut self) -> u8 {
        unsafe {
            wait_for!(self.line_sts().contains(LineStsFlags::INPUT_FULL));
            self.data.read()
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}

/// 通过串口输出到宿主机
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::utils::serial::serial_print(format_args!($($arg)*));
    };
}

/// 通过串口输出到宿主机
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(
        concat!($fmt, "\n"), $($arg)*));
}

/// 颜色字符串
#[macro_export]
macro_rules! color_text {
    ($text:expr, $color:expr) => {{
        format_args!("\x1b[{}m{}\x1b[0m", $color, $text)
    }};
}
