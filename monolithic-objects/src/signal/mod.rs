//! Signal related objects.
use crate::sync::Event;
use crate::task::Process;
pub use Signal::*;
pub use action::*;
use alloc::sync::Arc;
use bitflags::bitflags;
use hal::arch::cpu::MachineContext;
use log::info;
use num::FromPrimitive;
use num_derive::FromPrimitive;
use spin::Mutex;

mod action;

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, FromPrimitive)]
pub enum Signal {
    SIGHUP = 1,
    SIGINT = 2,
    SIGQUIT = 3,
    SIGILL = 4,
    SIGTRAP = 5,
    SIGABRT = 6,
    SIGBUS = 7,
    SIGFPE = 8,
    SIGKILL = 9,
    SIGUSR1 = 10,
    SIGSEGV = 11,
    SIGUSR2 = 12,
    SIGPIPE = 13,
    SIGALRM = 14,
    SIGTERM = 15,
    SIGSTKFLT = 16,
    SIGCHLD = 17,
    SIGCONT = 18,
    SIGSTOP = 19,
    SIGTSTP = 20,
    SIGTTIN = 21,
    SIGTTOU = 22,
    SIGURG = 23,
    SIGXCPU = 24,
    SIGXFSZ = 25,
    SIGVTALRM = 26,
    SIGPROF = 27,
    SIGWINCH = 28,
    SIGIO = 29,
    SIGPWR = 30,
    SIGSYS = 31,
    // real time signals
    SIGRT32 = 32,
    SIGRT33 = 33,
    SIGRT34 = 34,
    SIGRT35 = 35,
    SIGRT36 = 36,
    SIGRT37 = 37,
    SIGRT38 = 38,
    SIGRT39 = 39,
    SIGRT40 = 40,
    SIGRT41 = 41,
    SIGRT42 = 42,
    SIGRT43 = 43,
    SIGRT44 = 44,
    SIGRT45 = 45,
    SIGRT46 = 46,
    SIGRT47 = 47,
    SIGRT48 = 48,
    SIGRT49 = 49,
    SIGRT50 = 50,
    SIGRT51 = 51,
    SIGRT52 = 52,
    SIGRT53 = 53,
    SIGRT54 = 54,
    SIGRT55 = 55,
    SIGRT56 = 56,
    SIGRT57 = 57,
    SIGRT58 = 58,
    SIGRT59 = 59,
    SIGRT60 = 60,
    SIGRT61 = 61,
    SIGRT62 = 62,
    SIGRT63 = 63,
    SIGRT64 = 64,
}

impl Signal {
    pub const RTMIN: usize = 32;
    pub const RTMAX: usize = 64;

    pub fn is_standard(self) -> bool {
        (self as usize) < Self::RTMIN
    }
}

/// Send signal to process.
pub fn send_signal(process: Arc<Mutex<Process>>, tid: isize, info: Siginfo) {
    let signal: Signal = FromPrimitive::from_i32(info.signo).unwrap();
    let mut process = process.lock();
    if signal.is_standard() && process.pending_sigset.contains(signal) {
        return;
    }
    process.signals.push_back((info, tid));
    process.pending_sigset.add(signal);
    process.eventbus.lock().set(Event::RECEIVE_SIGNAL);
    info!(
        "send signal {} to pid {} tid {}",
        info.signo, process.pid, tid
    )
}

bitflags! {
    pub struct SignalStackFlags : u32 {
        /// Using the stack.
        const ONSTACK = 1;
        /// Stack disabled.
        const DISABLE = 2;
        /// Auto disable stack.
        const AUTODISARM = 0x80000000;
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SignalStack {
    pub sp: usize,
    pub flags: u32,
    pub size: usize,
}

impl Default for SignalStack {
    fn default() -> Self {
        // default to disabled
        SignalStack {
            sp: 0,
            flags: SignalStackFlags::DISABLE.bits(),
            size: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct SignalFrame {
    /// ret addr. Point ro ret_code.
    pub ret_code_addr: usize,
    /// Signal info.
    pub info: Siginfo,
    /// Usercontext.
    pub ucontext: SignalUserContext,
    /// Execute this when done signal handling.
    /// Default set to sys_rt_sigreturn().
    pub ret_code: [u8; 7],
}

/// See musl struct __ucontext
/// Not exactly the same for now
#[repr(C)]
#[derive(Clone)]
pub struct SignalUserContext {
    pub flags: usize,
    pub link: usize,
    pub stack: SignalStack,
    pub context: MachineContext,
    pub sig_mask: Sigset,
}
