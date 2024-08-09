//! 一些调试用的系统调用
use crate::fs::ROOT_INODE;
use crate::print;
use crate::println;
use crate::serial::serial_receive;
use alloc::string::String;

/// 输出用户态内容
///
/// TODO: 将用户和内核的数据传输封装起来提高安全性
pub fn sys_debug_write(msg_ptr: usize) -> (usize, usize) {
    let msg_ptr = msg_ptr as *const &str;
    let msg = unsafe { *(msg_ptr) };
    print!("{}", msg);
    (msg.len(), 0)
}

/// 从串口读入一个字节并写入用户态buf
pub fn sys_serial_read(buf_ptr: usize) -> (usize, usize) {
    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, 1) };
    let char = serial_receive();
    buf[0] = char;
    (1, char as _)
}

/// 用户态数据传输
pub fn sys_debug_data_transport(bufs_ptr: usize, ret_ptr: usize) -> (usize, usize) {
    let bufs_ptr = bufs_ptr as *const &[&[u8]];
    let buffers = unsafe { *bufs_ptr };
    // let mut buffer_vector: Vec<Arc<DataBuffer>> = Vec::with_capacity(buffers.len());
    for &buffer in buffers {
        println!("[In sys_debug_transport] buf_ptr = {:#x?}", buffer.as_ptr());
        let buffer = buffer.to_vec();
        let read_str = String::from_utf8(buffer).ok();
        println!("Kernel read str {}", read_str.unwrap());
    }

    let kernel_str = "This is str form kernel!".as_bytes();
    let ret_ptr = ret_ptr as *mut u8;
    unsafe {
        ret_ptr.copy_from_nonoverlapping(kernel_str.as_ptr(), kernel_str.len());
    }
    (0, 0)
}

/// 测试是否能通过ROOT_INODE查找
pub fn sys_debug_open(name_ptr: usize) -> (usize, usize) {
    let name_ptr = name_ptr as *const &str;
    let name = unsafe { *name_ptr };
    println!("[In sys_debug_open] Kernel received: {}", name);
    let inode = ROOT_INODE.find(name);
    assert!(inode.is_some());
    (0, 0)
}
