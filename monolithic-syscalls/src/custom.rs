//! 自定义系统调用（用于Debug）
use super::*;
use hal::check_n_clone_cstr_array;
use user_syscall::SysResult;

impl Syscall<'_> {
    pub fn sys_test_cstr(&self, ptr: *const *const u8) -> SysResult {
        let args = check_n_clone_cstr_array(ptr)?;
        info!("args = {:?}", args);
        Ok(0)
    }
}
