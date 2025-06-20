//! 自定义系统调用

use alloc::vec::Vec;

use crate::{sys_test_cstr, SysResult};
pub fn test_cstr(args: &[&str]) -> SysResult {
    let ptrs: Vec<*const u8> = args.iter().map(|&s| s.as_ptr()).collect();
    sys_test_cstr(ptrs.as_ptr())
}
