//! 任务管理类系统调用
use crate::*;

impl Syscall<'_> {
    /// Fork current process, return child's PID.
    pub fn sys_fork(&mut self) -> SysResult {
        let new_thread = self.thread.fork(self.context);
    }
}