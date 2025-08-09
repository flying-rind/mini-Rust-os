//! SigAction.

use super::Signal;
use bitflags::bitflags;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Siginfo {
    pub signo: i32,
    pub errno: i32,
    pub code: i32,
    pub field: SiginfoFields,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union SiginfoFields {
    pad: [u8; Self::PAD_SIZE],
    // TODO: fill this union
}

impl SiginfoFields {
    const PAD_SIZE: usize = 128 - 2 * core::mem::size_of::<i32>() - core::mem::size_of::<usize>();
}

/// sigset_t
#[derive(Default, Clone, Copy, Debug)]
#[repr(C)]
pub struct Sigset(u64);

impl Sigset {
    pub fn empty() -> Self {
        Sigset(0)
    }

    pub fn contains(&self, sig: Signal) -> bool {
        (self.0 >> sig as u64 & 1) != 0
    }

    pub fn add(&mut self, sig: Signal) {
        self.0 |= 1 << sig as u64;
    }
    pub fn add_set(&mut self, sigset: &Sigset) {
        self.0 |= sigset.0;
    }
    pub fn remove(&mut self, sig: Signal) {
        self.0 ^= self.0 & (1 << sig as u64);
    }
    pub fn remove_set(&mut self, sigset: &Sigset) {
        self.0 ^= self.0 & sigset.0;
    }

    pub fn mask_with(&self, mask: &Sigset) -> Sigset {
        Sigset(self.0 & (!mask.0))
    }
}

/// Sigactions.
pub struct SignalActions {
    pub table: [SignalAction; Signal::RTMAX + 1],
}

impl Default for SignalActions {
    fn default() -> Self {
        Self {
            table: [SignalAction::default(); Signal::RTMAX + 1],
        }
    }
}

/// Linux struct sigaction
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SignalAction {
    /// Signal handler.
    pub handler: usize, // this field may be an union
    /// Flags when handle signal.
    pub flags: SignalActionFlags,
    /// ?
    pub restorer: usize,
    /// Mask when handle signal.
    pub mask: Sigset,
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy)]
    pub struct SignalActionFlags : usize {
        const NOCLDSTOP = 1;
        const NOCLDWAIT = 2;
        const SIGINFO = 4;
        const ONSTACK = 0x08000000;
        const RESTART = 0x10000000;
        const NODEFER = 0x40000000;
        const RESETHAND = 0x80000000;
        const RESTORER = 0x04000000;
    }
}
