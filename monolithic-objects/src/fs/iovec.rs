//! IoVec Struct used by writev.
use alloc::vec;
use alloc::vec::Vec;
use hal::SysError;
use log::info;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct IoVec {
    /// Starting address.
    base: *mut u8,
    /// Num of bytes.
    len: usize,
}

/// A valid IoVecs request from user, used by kernel.
#[derive(Debug)]
pub struct IoVecs(Vec<&'static mut [u8]>);

impl IoVecs {
    /// Create new IoVecs.
    pub unsafe fn new(iov_ptr: *const IoVec, iov_cnt: usize) -> Result<Self, SysError> {
        // FIXME: Check first.
        let iovs = unsafe { core::slice::from_raw_parts(iov_ptr, iov_cnt).to_vec() };
        let mut slices = vec![];
        slices.reserve(iovs.len());
        for iov in iovs.iter() {
            // Just for now.
            if iov.base.is_null() || iov.len == 0 {
                continue;
            }
            info!("Iov base: {:x?}, iov len: {}", iov.base, iov.len);
            unsafe {
                slices.push(core::slice::from_raw_parts_mut(iov.base, iov.len));
            }
        }
        Ok(IoVecs(slices))
    }

    /// Read all to a vec.
    pub fn read_all_to_vec(&self) -> Vec<u8> {
        let mut buf = self.new_buf(false);
        for slice in self.0.iter() {
            buf.extend(slice.iter());
        }
        buf
    }

    /// Create a new Vec buffer from IoVecs
    /// For readv:  `set_len` is true,  Vec.len = total_len.
    /// For writev: `set_len` is false, Vec.cap = total_len.
    pub fn new_buf(&self, set_len: bool) -> Vec<u8> {
        let total_len = self.0.iter().map(|slice| slice.len()).sum::<usize>();
        let mut buf = Vec::with_capacity(total_len);
        if set_len {
            unsafe {
                buf.set_len(total_len);
            }
        }
        buf
    }
}
