//! 系统调用处理函数和中断处理函数
use super::*;

// 系统调用处理函数
#[no_mangle]
pub extern "C" fn syscall_handler(f: &'static mut SyscallFrame) -> isize {
    let r = &f.caller;
    let ret = syscall::syscall(r.rax, [r.rdi, r.rsi, r.rdx], f);
    // current_check_signal();
    ret
}
