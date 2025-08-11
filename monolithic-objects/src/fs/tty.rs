//! tty.
//! Copied from rCore.

#![allow(unused)]
use crate::Pgid;
use crate::process_group;
use crate::sync::Event;
use crate::sync::EventBus;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use bitflags::bitflags;
use core::any::Any;
use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use hal::print;
use lazy_static::lazy_static;
use log::info;
use log::warn;
use rcore_fs::vfs::FsError::NotSupported;
use rcore_fs::vfs::*;
use spin::{Mutex, RwLock};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Winsize {
    row: u16,
    ws_col: u16,
    xpixel: u16,
    ypixel: u16,
}

// Ref: https://www.man7.org/linux/man-pages/man3/termios.3.html
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Termios {
    pub iflag: u32,
    pub oflag: u32,
    pub cflag: u32,
    pub lflag: u32,
    pub line: u8,
    pub cc: [u8; 32],
    pub ispeed: u32,
    pub ospeed: u32,
}

impl Default for Termios {
    fn default() -> Self {
        Termios {
            // IMAXBEL | IUTF8 | IXON | IXANY | ICRNL | BRKINT
            iflag: 0o66402,
            // OPOST | ONLCR
            oflag: 0o5,
            // HUPCL | CREAD | CSIZE | EXTB
            cflag: 0o2277,
            // IEXTEN | ECHOTCL | ECHOKE ECHO | ECHOE | ECHOK | ISIG | ICANON
            lflag: 0o105073,
            line: 0,
            cc: [
                3,   // VINTR Ctrl-C
                28,  // VQUIT
                127, // VERASE
                21,  // VKILL
                4,   // VEOF Ctrl-D
                0,   // VTIME
                1,   // VMIN
                0,   // VSWTC
                17,  // VSTART
                19,  // VSTOP
                26,  // VSUSP Ctrl-Z
                255, // VEOL
                18,  // VREPAINT
                15,  // VDISCARD
                23,  // VWERASE
                22,  // VLNEXT
                255, // VEOL2
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            ispeed: 0,
            ospeed: 0,
        }
    }
}

/// console tty
// Ref: [https://linux.die.net/man/4/tty]
#[derive(Default)]
pub struct TtyINode {
    /// foreground process group
    foreground_pgid: RwLock<Pgid>,
    buf: Mutex<VecDeque<u8>>,
    eventbus: Mutex<EventBus>,
    winsize: RwLock<Winsize>,
    termios: RwLock<Termios>,
}

// ref: https://www.man7.org/linux/man-pages/man3/termios.3.html
// c_lflag constants
bitflags! {
    #[derive(Debug)]
    pub struct LocalModes : u32 {
        const ISIG = 0o000001;
        const ICANON = 0o000002;
        const ECHO = 0o000010;
        const ECHOE = 0o000020;
        const ECHOK = 0o000040;
        const ECHONL = 0o000100;
        const NOFLSH = 0o000200;
        const TOSTOP = 0o000400;
        const IEXTEN = 0o100000;
        const XCASE = 0o000004;
        const ECHOCTL = 0o001000;
        const ECHOPRT = 0o002000;
        const ECHOKE = 0o004000;
        const FLUSHO = 0o010000;
        const PENDIN = 0o040000;
        const EXTPROC = 0o200000;
    }
}

lazy_static! {
    pub static ref TTY: Arc<TtyINode> = Arc::new(TtyINode::default());
}

pub fn foreground_pgid() -> Pgid {
    *TTY.foreground_pgid.read()
}

impl TtyINode {
    pub fn push(&self, c: u8) {
        let lflag = LocalModes::from_bits_truncate(self.termios.read().lflag);
        if lflag.contains(LocalModes::ISIG) && [0o3, 0o34, 0o32, 0o31].contains(&(c as i32)) {
            use crate::signal::*;
            let foregroud_processes = process_group(foreground_pgid());
            match c as i32 {
                // INTR
                0o3 => {
                    for proc in foregroud_processes {
                        send_signal(
                            proc,
                            -1,
                            Siginfo {
                                signo: SIGINT as i32,
                                errno: 0,
                                code: SI_KERNEL,
                                field: Default::default(),
                            },
                        );
                    }
                }
                _ => warn!("special char {} is unimplented", c),
            }
        } else {
            self.buf.lock().push_back(c);
            self.eventbus.lock().set(Event::READABLE);
        }
    }

    pub fn pop(&self) -> u8 {
        let mut buf_lock = self.buf.lock();
        let c = buf_lock.pop_front().unwrap();
        if buf_lock.len() == 0 {
            self.eventbus.lock().clear(Event::READABLE);
        }
        return c;
    }
    pub fn can_read(&self) -> bool {
        return self.buf.lock().len() > 0;
    }
}

impl INode for TtyINode {
    /// Read bytes at `offset` into `buf`, return the number of bytes read.
    fn read_at(&self, _offset: usize, buf: &mut [u8]) -> Result<usize> {
        if self.can_read() {
            buf[0] = self.pop() as u8;
            Ok(1)
        } else {
            Err(FsError::Again)
        }
    }

    /// Write bytes at `offset` from `buf`, return the number of bytes written.
    fn write_at(&self, _offset: usize, buf: &[u8]) -> Result<usize> {
        use core::str;
        // we do not care the utf-8 things, we just want to print it!
        let s = unsafe { str::from_utf8_unchecked(buf) };
        print!("{}", s);
        Ok(buf.len())
    }

    /// Poll the events, return a bitmap of events.
    fn poll(&self) -> Result<PollStatus> {
        Ok(PollStatus {
            read: self.can_read(),
            write: true,
            error: false,
        })
    }

    fn async_poll<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<PollStatus>> + Send + Sync + 'a>> {
        #[must_use = "future does nothing unless polled/`await`-ed"]
        struct SerialFuture<'a> {
            tty: &'a TtyINode,
        }

        impl<'a> Future for SerialFuture<'a> {
            type Output = Result<PollStatus>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
                if self.tty.can_read() {
                    return Poll::Ready(self.tty.poll());
                }
                let waker = cx.waker().clone();
                self.tty.eventbus.lock().subscribe(Box::new({
                    move |_| {
                        waker.wake_by_ref();
                        true
                    }
                }));
                Poll::Pending
            }
        }

        Box::pin(SerialFuture { tty: self })
    }

    fn io_control(&self, cmd: u32, data: usize) -> Result<usize> {
        let cmd = cmd as usize;
        pub const TIOCGPGRP: usize = 0x540F;
        pub const TIOCSPGRP: usize = 0x5410;
        pub const TCGETS: usize = 0x5401;
        pub const TCSETS: usize = 0x5402;
        pub const TIOCGWINSZ: usize = 0x5413;

        match cmd {
            TIOCGPGRP => {
                // TODO: check the pointer?
                let argp = data as *mut i32; // pid_t
                unsafe { *argp = *self.foreground_pgid.read() };
                Ok(0)
            }
            TIOCSPGRP => {
                let fpgid = unsafe { *(data as *const i32) };
                *self.foreground_pgid.write() = fpgid;
                info!("tty: set foreground process group to {}", fpgid);
                Ok(0)
            }
            TIOCGWINSZ => {
                let winsize = data as *mut Winsize;
                unsafe {
                    *winsize = *self.winsize.read();
                }
                Ok(0)
            }
            TCGETS => {
                let termois = data as *mut Termios;
                unsafe {
                    *termois = *self.termios.read();
                }
                let lflag = LocalModes::from_bits_truncate(self.termios.read().lflag);
                info!("get lfags: {:?}", lflag);
                Ok(0)
            }
            TCSETS => {
                let termois = data as *const Termios;
                unsafe {
                    *self.termios.write() = *termois;
                }
                let lflag = LocalModes::from_bits_truncate(self.termios.read().lflag);
                info!("set lfags: {:?}", lflag);
                Ok(0)
            }
            _ => Err(NotSupported),
        }
    }

    /// Get metadata of the INode
    fn metadata(&self) -> Result<Metadata> {
        Ok(Metadata {
            dev: 1,
            inode: 13,
            size: 0,
            blk_size: 0,
            blocks: 0,
            atime: Timespec { sec: 0, nsec: 0 },
            mtime: Timespec { sec: 0, nsec: 0 },
            ctime: Timespec { sec: 0, nsec: 0 },
            type_: FileType::CharDevice,
            mode: 0o666,
            nlinks: 1,
            uid: 0,
            gid: 0,
            rdev: make_rdev(5, 0),
        })
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}
