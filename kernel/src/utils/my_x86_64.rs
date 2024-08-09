use core::arch::asm;

#[inline(always)]
pub fn read_rsp() -> usize {
    let val: usize;
    unsafe {
        asm!("mov {}, rsp", out(reg) val);
    }
    val
}

#[inline(always)]
pub fn in8(port: u16) -> u8 {
    let val: u8;
    unsafe {
        asm!("in al, dx", out("al") val, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    val
}

#[inline(always)]
pub fn in16(port: u16) -> u16 {
    let val: u16;
    unsafe {
        asm!("in ax, dx", out("ax") val, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    val
}

#[inline(always)]
pub fn in32(port: u32) -> u32 {
    let val: u32;
    unsafe {
        asm!("in eax, dx", out("eax") val, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    val
}

#[inline(always)]
pub fn out8(port: u16, val: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn out16(port: u16, val: u16) {
    unsafe {
        asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn out32(port: u32, val: u32) {
    unsafe {
        asm!("out dx, eax", in("dx") port, in("eax") val, options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn disable_interrupts() {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
}

#[inline(always)]
pub fn enable_interrupts_and_hlt() {
    unsafe {
        asm!("sti; hlt", options(nomem, nostack));
    }
}

pub const RING0: u16 = 0;
pub const RING3: u16 = 3;

pub const RFLAGS_IF: usize = 1 << 9;

#[inline(always)]
pub fn get_msr(id: u32) -> usize {
    let (high, low): (u32, u32);
    unsafe {
        asm!("rdmsr", in("ecx") id, out("eax") low, out("edx") high, options(nomem, nostack, preserves_flags));
    }
    ((high as usize) << 32) | (low as usize)
}

#[inline(always)]
pub fn set_msr(id: u32, val: usize) {
    let low = val as u32;
    let high = (val >> 32) as u32;
    unsafe {
        asm!("wrmsr", in("ecx") id, in("eax") low, in("edx") high, options(nostack, preserves_flags));
    }
}

pub const EFER_MSR: u32 = 0xC000_0080;
pub const STAR_MSR: u32 = 0xC000_0081;
pub const LSTAR_MSR: u32 = 0xC000_0082;
pub const SFMASK_MSR: u32 = 0xC000_0084;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct DescriptorTablePointer {
    /// Size of the DT.
    pub limit: u16,
    /// Pointer to the memory region containing the DT.
    pub base: usize,
}

/// Load a GDT.
#[inline(always)]
pub fn lgdt(gdt: &DescriptorTablePointer) {
    unsafe {
        asm!("lgdt [{}]", in(reg) gdt, options(readonly, nostack, preserves_flags));
    }
}

/// Load an IDT.
#[inline(always)]
pub fn lidt(idt: &DescriptorTablePointer) {
    unsafe {
        asm!("lidt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
    }
}

/// Load the task state register using the `ltr` instruction.
#[inline(always)]
pub fn load_tss(sel: u16) {
    unsafe {
        asm!("ltr {0:x}", in(reg) sel, options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn set_cs(sel: u16) {
    unsafe {
        asm!(
          "push {sel}",
          "lea {tmp}, [1f + rip]",
          "push {tmp}",
          "retfq",
          "1:",
          sel = in(reg) sel as usize,
          tmp = lateout(reg) _,
          options(preserves_flags),
        );
    }
}

#[inline(always)]
pub fn set_ss(sel: u16) {
    unsafe {
        asm!("mov ss, {0:x}", in(reg) sel, options(nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn get_cr3() -> usize {
    let val: usize;
    unsafe {
        asm!("mov {}, cr3", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    // Mask top bits and flags.
    val & 0x_000f_ffff_ffff_f000
}

#[inline(always)]
pub fn set_cr3(pa: usize) {
    unsafe {
        asm!("mov cr3, {}", in(reg) pa, options(nostack, preserves_flags));
    }
}

bitflags::bitflags! {
    /// 构架特定的页表项标识
    #[derive(Default)]
    pub struct PageTableFlags: usize {
        /// Specifies whether the mapped frame or page table is loaded in memory.
        const PRESENT =         1 << 0;
        /// Controls whether writes to the mapped frames are allowed.
        ///
        /// If this bit is unset in a level 1 page table entry, the mapped frame is read-only.
        /// If this bit is unset in a higher level page table entry the complete range of mapped
        /// pages is read-only.
        const WRITABLE =        1 << 1;
        /// Controls whether accesses from userspace (i.e. ring 3) are permitted.
        const USER_ACCESSIBLE = 1 << 2;
        /// If this bit is set, a “write-through” policy is used for the cache, else a “write-back”
        /// policy is used.
        const WRITE_THROUGH =   1 << 3;
        /// Disables caching for the pointed entry is cacheable.
        const NO_CACHE =        1 << 4;
        /// Set by the CPU when the mapped frame or page table is accessed.
        const ACCESSED =        1 << 5;
        /// Set by the CPU on a write to the mapped frame.
        const DIRTY =           1 << 6;
        /// Specifies that the entry maps a huge frame instead of a page table. Only allowed in
        /// P2 or P3 tables.
        const HUGE_PAGE =       1 << 7;
        /// Indicates that the mapping is present in all address spaces, so it isn't flushed from
        /// the TLB on an address space switch.
        const GLOBAL =          1 << 8;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_9 =           1 << 9;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_10 =          1 << 10;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_11 =          1 << 11;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_52 =          1 << 52;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_53 =          1 << 53;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_54 =          1 << 54;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_55 =          1 << 55;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_56 =          1 << 56;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_57 =          1 << 57;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_58 =          1 << 58;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_59 =          1 << 59;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_60 =          1 << 60;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_61 =          1 << 61;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_62 =          1 << 62;
        /// Forbid code execution from the mapped frames.
        ///
        /// Can be only used when the no-execute page protection feature is enabled in the EFER
        /// register.
        const NO_EXECUTE =      1 << 63;
    }
}
