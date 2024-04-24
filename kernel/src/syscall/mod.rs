const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
// const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

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
mod uaccess;

use crate::{trap::SyscallFrame, *};
use fs::*;
use proc::*;

pub use uaccess::*;

pub fn syscall(syscall_id: usize, args: [usize; 3], f: &mut SyscallFrame) -> isize {
    let ret = match syscall_id {
        SYSCALL_OPEN => sys_open(args[0] as _, args[1] as _),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_READ => sys_read(args[0], args[1] as _, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as _, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_FORK => sys_fork(f),
        SYSCALL_EXEC => sys_exec(args[0] as _, f),
        SYSCALL_WAITPID => sys_waitpid(args[0] as _, args[1] as _),
        _ => {
            println!("Unsupported syscall: {}", syscall_id);
            process::current().exit(-1);
        }
    };
    ret
}
