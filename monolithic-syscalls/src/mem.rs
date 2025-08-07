//! Memory related syscalls.

use crate::Syscall;

impl Syscall<'_> {
    ///
    pub fn sys_mmap(
        &mut self,
        addr: usize,
        len: usize,
        prot: usize,
        flags: usize,
        fd: usize,
        offset: usize,
    ) {
    }
}
