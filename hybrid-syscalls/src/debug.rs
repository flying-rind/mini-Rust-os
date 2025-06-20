//! 一些调试用的系统调用
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Debug;
use hal::console::serial_receive;
use hybrid_objects::fs::ROOT_INODE;
use hybrid_objects::mm::PHYS_OFFSET;
use hybrid_objects::print;
use hybrid_objects::println;
use log::{error, info};

#[derive(Debug)]
pub enum SysError {
    EFAULT = 0,
}

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
    assert!(inode.is_ok());
    (0, 0)
}

pub fn sys_test_cstr(ptr: *const *const u8) -> (usize, usize) {
    let args = check_n_clone_cstr_array(ptr).expect("test_cstr failed!");
    info!("args = {:?}", args);
    (0, 0)
}

/// 检查并复制C语言字符串
/// FIXME:Move to HAL
pub fn check_n_clone_cstr(user: *const u8) -> Result<String, SysError> {
    if user.is_null() {
        Ok(String::new())
    } else {
        let mut buffer = Vec::new();
        for i in 0.. {
            let addr = unsafe { user.add(i) };
            let data = copy_from_user(addr).ok_or(SysError::EFAULT)?;
            if data == 0 {
                break;
            }
            buffer.push(data);
        }
        String::from_utf8(buffer).map_err(|_| SysError::EFAULT)
    }
}

/// 检查并复制多个C字符串
/// FIXME:Move to HAL
pub fn check_n_clone_cstr_array(user: *const *const u8) -> Result<Vec<String>, SysError> {
    if user.is_null() {
        Ok(Vec::new())
    } else {
        let mut buffer = Vec::new();
        for i in 0.. {
            let addr = unsafe { user.add(i) };
            let str_ptr = copy_from_user(addr).ok_or(SysError::EFAULT)?;
            if str_ptr.is_null() {
                break;
            }
            let string = check_n_clone_cstr(str_ptr)?;
            buffer.push(string);
        }
        Ok(buffer)
    }
}

/// 从用户态复制到内核
/// FIXME:Move to HAL
pub fn copy_from_user<T: Debug>(addr: *const T) -> Option<T> {
    #[inline(never)]
    unsafe extern "C" fn read_user<T>(dst: *mut T, src: *const T) -> usize {
        unsafe {
            dst.copy_from_nonoverlapping(src, 1);
        }
        0
    }
    if !access_ok(addr as usize, size_of::<T>()) {
        return None;
    }
    let mut dst: T = unsafe { core::mem::zeroed() };
    match unsafe { read_user(&mut dst as *mut T, addr) } {
        0 => Some(dst),
        _ => None,
    }
}

/// Check whether the address rage [addr, addr + len) is not in kernel space
/// FIXME:Move to HAL
pub fn access_ok(addr: usize, len: usize) -> bool {
    addr < PHYS_OFFSET && (addr + len) < PHYS_OFFSET
}
