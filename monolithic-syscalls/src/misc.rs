//! Misc syscalls
use super::*;
use hal::SysError;
use user_syscall::SysResult;

impl Syscall<'_> {
    /// Currently only supprot ARCH_SET_FS.(For musl-libc?)
    /// FIXME: Implement all ops.
    ///
    /// [arch_prctl](https://man7.org/linux/man-pages/man2/arch_prctl.2.html)
    pub fn sys_arch_prctl(&mut self, code: i32, addr: usize) -> SysResult {
        const ARCH_SET_FS: i32 = 0x1002;
        match code {
            ARCH_SET_FS => {
                info!("sys_arch_prctl: set FSBASE to {:#x}", addr);
                self.context.general.fsbase = addr;
                Ok(0)
            }
            _ => Err(SysError::EINVAL),
        }
    }
}
