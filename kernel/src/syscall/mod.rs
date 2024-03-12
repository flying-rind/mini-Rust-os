//! 系统调用号和总控函数
use self::process::*;
use crate::process::*;

const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
// const SYSCALL_GET_TIME: usize = 169;
// const SYSCALL_GETPID: usize = 172;
// const SYSCALL_FORK: usize = 220;
// const SYSCALL_EXEC: usize = 221;
// const SYSCALL_WAITPID: usize = 260;
const EFAULT: isize = -1;

mod fs;
mod process;
mod uaccess;

use fs::*;
pub use uaccess::*;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    let ret = match syscall_id {
        SYSCALL_READ => sys_read(args[0], args[1] as _, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as _, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        _ => {
            serial_println!("Unsupported syscall: {}", syscall_id);
            current_exit(-1);
        }
    };
    ret
}
