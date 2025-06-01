//! 任务管理类系统调用
use crate::*;

impl Syscall<'_> {
    /// Fork current process, return child's PID.
    pub fn sys_fork(&mut self) -> SysResult {
        let new_thread = self.thread.fork(self.context);
        let pid = new_thread.proc.lock().pid.0;
        let future = (self.thread_fn)(self.thread.clone());
        executor::spawn(future);
        Ok(pid)
    }
}