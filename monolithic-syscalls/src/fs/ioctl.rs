//! ioctl syscall related consts.

pub const TCGETS: usize = 0x5401;

pub const TCSETS: usize = 0x5402;

pub const TIOCGPGRP: usize = 0x540F;

pub const TIOCSPGRP: usize = 0x5410;

pub const TIOCGWINSZ: usize = 0x5413;

pub const FIONCLEX: usize = 0x5450;

pub const FIOCLEX: usize = 0x5451;
