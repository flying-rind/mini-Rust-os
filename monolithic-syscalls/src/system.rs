//! System related syscalls.
use super::*;

impl Syscall<'_> {
    /// uname() returns system information in the structure pointed to by
    /// buf.  The utsname struct is defined in <sys/utsname.h>:
    ///
    /// struct utsname {
    ///        char sysname[];    /* Operating system name (e.g., "Linux") */
    ///        char nodename[];   /* Name within communications network
    ///                              to which the node is attached, if any */
    ///        char release[];    /* Operating system release
    ///                              (e.g., "2.6.28") */
    ///        char version[];    /* Operating system version */
    ///        char machine[];    /* Hardware type identifier */
    ///    #ifdef _GNU_SOURCE
    ///        char domainname[]; /* NIS or YP domain name */
    ///    #endif
    ///  };
    ///
    /// [uname(2)](https://man7.org/linux/man-pages/man2/uname.2.html)
    pub fn sys_uname(&mut self, buf: *mut u8) -> SysResult {
        info!("uname: buf: {:?}", buf);

        let offset = 65;
        let strings = ["Linux", "orz", "0.1.0", "1", "x86_64", "domain"];
        let buf = unsafe { core::slice::from_raw_parts_mut(buf, offset * strings.len()) };

        for i in 0..strings.len() {
            unsafe {
                hal::write_cstr(&mut buf[i * offset], &strings[i]);
            }
        }
        Ok(0)
    }
}
