//! Sync related syscalls.

use core::sync::atomic::{AtomicI32, Ordering};

use hal::{SysError, user::UserInPtr};
use log::warn;
use monolithic_objects::sync::TimeSpec;
use user_syscall::SysResult;

use super::*;

impl Syscall<'_> {
    /// The futex() system call provides a method for waiting until a
    /// certain condition becomes true.  It is typically used as a
    /// blocking construct in the context of shared-memory
    /// synchronization.  When using futexes, the majority of the
    /// synchronization operations are performed in user space.  A user-
    /// space program employs the futex() system call only when it is
    /// likely that the program has to block for a longer time until the
    /// condition becomes true.  Other futex() operations can be used to
    /// wake any processes or threads waiting for a particular condition.
    ///
    /// [futex(man2)](https://man7.org/linux/man-pages/man2/futex.2.html)
    ///
    /// FIXME: Currently only support private ops(wait and wake).
    pub async fn sys_futex(
        &mut self,
        uaddr: usize,
        op: u32,
        val: i32,
        timeout: UserInPtr<TimeSpec>,
    ) -> SysResult {
        const OP_PRIVATE: u32 = 0x80;
        const OP_WAIT: u32 = 0;
        const OP_WAKE: u32 = 1;
        info!(
            "futex: [{}] uaddr: {:#x}, op: {:#x}, val: {}, timeout_ptr: {:?}",
            self.thread.tid, uaddr, op, val, timeout
        );
        if op & OP_PRIVATE == 0 {
            warn!("process-shared futex is unimplemented");
            unimplemented!("Not implemented!");
        }
        // uaddr must be 4 bytes aligned.
        if uaddr % size_of::<u32>() != 0 {
            return Err(SysError::EINVAL);
        }
        let atomic: &'static AtomicI32 =
            unsafe { &core::slice::from_raw_parts_mut(uaddr as *mut AtomicI32, 1)[0] };

        let mut proc = self.process();
        let futex = proc.get_futex(uaddr);
        drop(proc);

        match op & 0xf {
            OP_WAIT => {
                if atomic.load(Ordering::Acquire) != val {
                    return Err(SysError::EAGAIN);
                }
                if timeout.is_null() {
                    futex.wait(None).await?;
                    Ok(0)
                } else {
                    let timeout = timeout.read()?;
                    info!("futex wait timeout: {:?}", timeout);
                    futex.wait(Some(timeout.to_duration())).await?;
                    return Ok(0);
                }
            }
            OP_WAKE => {
                let woken_up_count = futex.wake(val as usize);
                return Ok(woken_up_count);
            }
            _ => {
                warn!("unsupported futex operation: {}", op);
                unimplemented!("Not supported yet.");
            }
        }
    }
}
