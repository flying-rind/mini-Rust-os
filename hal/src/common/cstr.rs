//! C语言字符串操作

use crate::PHYS_OFFSET;
use crate::SysError;
use alloc::string::String;
use alloc::vec::Vec;

/// 检查并复制C语言字符串
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
pub fn copy_from_user<T>(addr: *const T) -> Option<T> {
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
