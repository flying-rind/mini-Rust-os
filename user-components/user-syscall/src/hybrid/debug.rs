//! 调试类系统调用
use alloc::vec::Vec;

use super::*;

/// 输出缓冲区

/// 打印到终端
pub fn debug_write(msg: &str) -> usize {
    let msg_ptr = &msg as *const &str as usize;
    let (ret0, _) = sys_debug_write(msg_ptr);
    ret0
}

/// 测试用户到内核的数据传输
pub fn debug_data_transport(buffers: &[&[u8]], result: &mut [u8]) -> usize {
    let bufs_ptr = &buffers as *const &[&[u8]] as usize;
    let ret_ptr = result.as_mut_ptr() as usize;
    let (ret0, _ret1) = sys_debug_data_transport(bufs_ptr, ret_ptr);
    ret0
}

/// 测试是否能使用内核全局ROOT_INODE查找文件系统
pub fn debug_open(name: &str) -> usize {
    let name_ptr = &name as *const &str as usize;
    let (ret0, _ret1) = sys_debug_open(name_ptr);
    ret0
}

/// 从串口读取一个字符
pub fn serial_read(buf: &mut [u8]) -> usize {
    // let buf_ptr = buf.as_mut_ptr();
    let (ret0, _ret1) = sys_serial_read(buf.as_mut_ptr() as usize);
    ret0
}

/// 获取时间
pub fn get_time() -> usize {
    let (ret0, _ret1) = sys_get_time();
    ret0
}

/// 测试用户到内核传递字符串数组
pub fn test_cstr(strs: &[&str]) -> bool {
    let str_ptrs: Vec<*const u8> = strs.iter().map(|&s| s.as_ptr()).collect();
    let ptr = str_ptrs.as_ptr();
    sys_test_cstr(ptr);
    true
}
