//! Signal related syscalls.
use hal::user::{UserInPtr, UserOutPtr};
use monolithic_objects::{SignalFrame, Sigset};

use super::*;

impl Syscall<'_> {
    /// This sigreturn() call undoes everything that was done—changing the
    /// process's signal mask, switching signal stacks (see
    /// sigaltstack(2))—in order to invoke the signal handler.
    ///
    /// [sigreturn2](https://man7.org/linux/man-pages/man2/sigreturn.2.html)
    pub fn sys_rt_sigreturn(&mut self) -> SysResult {
        info!("rt_sigreturn");
        let ptr: UserInPtr<SignalFrame> = UserInPtr::from(self.context.get_sp() - 8);
        let frame = ptr.read()?;

        let mut inner = self.thread.inner.lock();
        // restore altstack and sigmask.
        inner.signal_altstack = frame.ucontext.stack;
        inner.signal_mask = frame.ucontext.sig_mask;
        drop(inner);

        // restore context.
        frame.ucontext.context.fill_tf(self.context);
        Ok(self.context.get_syscall_ret())
    }

    /// sigprocmask() is used to fetch and/or change the signal mask of
    /// the calling thread.  The signal mask is the set of signals whose
    /// delivery is currently blocked for the caller (see also signal(7)
    /// for more details).
    ///
    /// [sys_rt_procmask(2)](https://man7.org/linux/man-pages/man2/sigprocmask.2.html)
    pub fn sys_rt_procmask(
        &mut self,
        how: usize,
        set: UserInPtr<Sigset>,
        mut oldset: UserOutPtr<Sigset>,
        size: usize,
    ) -> SysResult {
        info!(
            "rt_sigprocmask: how: {}, set: {:?}, oldset: {:?}, sigsetsize: {}",
            how, set, oldset, size
        );
        if size != 8 {
            return Err(SysError::EINVAL);
        }
        if !oldset.is_null() {
            oldset.write(self.thread.inner.lock().signal_mask)?;
        }
        if !set.is_null() {
            let set = set.read()?;
            const BLOCK: usize = 0;
            const UNBLOCK: usize = 1;
            const SETMASK: usize = 2;
            let mut inner = self.thread.inner.lock();
            match how {
                BLOCK => {
                    info!("rt_sigprocmask: block: {:x?}", set);
                    inner.signal_mask.add_set(&set);
                }
                UNBLOCK => {
                    info!("rt_sigprocmask: block: {:x?}", set);
                    inner.signal_mask.add_set(&set);
                }
                SETMASK => {
                    info!("rt_sigprocmask: block: {:x?}", set);
                    inner.signal_mask = set;
                }
                _ => return Err(SysError::EINVAL),
            }
        }
        return Ok(0);
    }
}
