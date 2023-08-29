use super::*;
use crate::my_x86_64::in8;
use crate::*;
use bitflags;

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;
const CHUNK_SIZE: usize = 256;

bitflags::bitflags! {
  struct LineSts: u8 {
    const INPUT_FULL = 1;
    const OUTPUT_EMPTY = 1 << 5;
  }
}

const SERIAL_DATA: u16 = 0x3F8;
const SERIAL_INT_EN: u16 = SERIAL_DATA + 1;
const SERIAL_FIFO_CTRL: u16 = SERIAL_DATA + 2;
const SERIAL_LINE_CTRL: u16 = SERIAL_DATA + 3;
const SERIAL_MODEM_CTRL: u16 = SERIAL_DATA + 4;
const SERIAL_LINE_STS: u16 = SERIAL_DATA + 5;

fn line_sts() -> LineSts {
    LineSts::from_bits_truncate(in8(SERIAL_LINE_STS))
}

pub fn receive() -> Option<u8> {
    if line_sts().contains(LineSts::INPUT_FULL) {
        Some(in8(SERIAL_DATA))
    } else {
        None
    }
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let mut n = 0;
            while n < len {
                let chunk_len = CHUNK_SIZE.min(len - n);
                let chunk = read_array::<CHUNK_SIZE>(unsafe { buf.add(n) }, chunk_len);
                if let Some(str) = chunk
                    .as_ref()
                    .and_then(|x| core::str::from_utf8(&x[..chunk_len]).ok())
                {
                    serial_print!("{}", str);
                    n += chunk_len;
                } else {
                    return EFAULT;
                }
            }
            n as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            assert_eq!(len, 1, "Only support len = 1 in sys_read!");
            loop {
                if let Some(c) = receive() {
                    return if buf.write_user(c).is_some() {
                        1
                    } else {
                        EFAULT
                    };
                } else {
                    crate::process::current_yield();
                }
            }
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}
