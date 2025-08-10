//! Memory related syscalls.

use crate::Syscall;
use bitflags::bitflags;
use hal::PAGE_SIZE;
use hybrid_objects::mm::{MemoryArea, PageTableFlags};
use log::info;
use user_syscall::SysResult;

impl Syscall<'_> {
    /// mmap() creates a new mapping in the virtual address space of the
    /// calling process.  The starting address for the new mapping is
    /// specified in addr.  The length argument specifies the length of
    /// the mapping (which must be greater than 0).
    pub fn sys_mmap(
        &mut self,
        addr: usize,
        len: usize,
        prot: usize,
        flags: usize,
        fd: usize,
        offset: usize,
    ) -> SysResult {
        let prot = MmapProt::from_bits_truncate(prot);
        let flags = MmapFlags::from_bits_truncate(flags);
        info!(
            "mmap: addr={:#x}, size={:#x}, prot={:?}, flags={:?}, fd={}, offset={:#x}",
            addr, len, prot, flags, fd as isize, offset
        );
        let proc = self.process();
        let mut addr = addr;
        if addr == 0 {
            // although NULL can be a valid address
            // but in C, NULL is regarded as allocation failure
            // so just skip it
            addr = PAGE_SIZE;
        }
        let vm = proc.vm.clone();
        // Must mapped at the exact addr.
        if flags.contains(MmapFlags::FIXED) {
            vm.remove_area(addr);
        // Kernel will find a area.
        } else {
            addr = vm.find_free_area(addr, len);
        }

        //
        if flags.contains(MmapFlags::ANONYMOUS) {
            let area = MemoryArea::new(addr, len, prot.to_flags());
            vm.insert_area(area);
            Ok(addr)
        } else {
            unimplemented!("File mmap unimplemented yet!");
        }
    }
}

bitflags! {
    #[derive(Debug)]
    pub struct MmapProt: usize {
        /// Data cannot be accessed
        const NONE = 0;
        /// Data can be read
        const READ = 1 << 0;
        /// Data can be written
        const WRITE = 1 << 1;
        /// Data can be executed
        const EXEC = 1 << 2;
    }
}

impl MmapProt {
    fn to_flags(self) -> PageTableFlags {
        let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
        if self.contains(MmapProt::WRITE) {
            flags |= PageTableFlags::WRITABLE;
        }
        flags
    }
}

bitflags! {
    #[derive(Debug)]
    pub struct MmapFlags: usize {
        /// Changes are shared.
        const SHARED = 1 << 0;
        /// Changes are private.
        const PRIVATE = 1 << 1;
        /// Place the mapping at the exact address
        const FIXED = 1 << 4;
        /// The mapping is not backed by any file. (non-POSIX)
        const ANONYMOUS = 1 << 5;
    }
}
