const SAYSCALL_DUP: usize = 24;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
// const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_THREAD_CREATE: usize = 1000;
const SYSCALL_GETTID: usize = 1001;
const SYSCALL_WAITTID: usize = 1002;
const SYSCALL_MUTEX_CREATE: usize = 1010;
const SYSCALL_MUTEX_LOCK: usize = 1011;
const SYSCALL_MUTEX_UNLOCK: usize = 1012;

const EFAULT: isize = -1;

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! try_ {
        ($x: expr, $err: expr) => {
            if let Some(x) = $x {
                x
            } else {
                return $err;
            }
        };
    }
}

mod fs;
mod proc;
mod sync;
mod uaccess;

use crate::*;
use fs::*;
use proc::*;
use sync::*;
pub use uaccess::*;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    let ret = match syscall_id {
        SAYSCALL_DUP => sys_dup(args[0] as _),
        SYSCALL_OPEN => sys_open(args[0] as _, args[1] as _),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_PIPE => sys_pipe(args[0] as _),
        SYSCALL_READ => sys_read(args[0], args[1] as _, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as _, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0] as _, args[1] as _),
        SYSCALL_WAITPID => sys_waitpid(args[0] as _, args[1] as _),
        SYSCALL_THREAD_CREATE => sys_thread_create(args[0], args[1]),
        SYSCALL_GETTID => sys_gettid(),
        SYSCALL_WAITTID => sys_waittid(args[0]),
        SYSCALL_MUTEX_CREATE => sys_mutex_create(args[0] == 1),
        SYSCALL_MUTEX_LOCK => sys_mutex_lock(args[0]),
        SYSCALL_MUTEX_UNLOCK => sys_mutex_unlock(args[0]),
        _ => {
            println!("Unsupported syscall: {}", syscall_id);
            task::current().exit(-1);
        }
    };
    ret
}
