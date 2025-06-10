//! 任务管理类系统调用
use super::*;

/// Copy current process, return child's PID.
pub fn fork() -> isize {
    sys_fork()
}
