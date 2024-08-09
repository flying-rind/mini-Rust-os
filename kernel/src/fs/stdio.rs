//! 定义标准输入输出，为其实现文件访问接口
use serial::serial_receive;

use super::File;
use crate::*;

/// 标准输入
pub struct Stdin;
/// 标准输出
pub struct Stdout;

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        false
    }

    /// 从串口读取一个字符到buf中
    fn read(&self, buf: &mut [u8]) -> usize {
        assert_eq!(buf.len(), 1);
        let c = serial_receive();
        buf[0] = c as _;
        return 1;
    }

    #[allow(unused)]
    fn write(&self, buf: &[u8]) -> usize {
        panic!("Cannot write to stdin!");
    }
}

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }

    fn writable(&self) -> bool {
        true
    }

    #[allow(unused)]
    fn read(&self, buf: &mut [u8]) -> usize {
        panic!("Cannot read from stdout!")
    }

    // 打印到串口（输出到主机屏幕）
    fn write(&self, buf: &[u8]) -> usize {
        if let Ok(str) = core::str::from_utf8(buf) {
            print!("{}", str);
            buf.len()
        } else {
            0
        }
    }
}
