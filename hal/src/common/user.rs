//! Raw pointer from user space.
use core::{fmt::Debug, marker::PhantomData};

use crate::{SysError, copy_from_user};

#[repr(C)]
/// Raw pointer from uspace.
pub struct UserPtr<T, P: Policy> {
    ptr: *mut T,
    // Why use this?
    mark: PhantomData<P>,
}

/// Indicate read or write
pub trait Policy {}
/// Can read
pub trait Read: Policy {}
/// Can write
pub trait Write: Policy {}

// Read only
pub enum In {}
// Write only
pub enum Out {}
/// Read and write
pub enum InOut {}

impl Policy for In {}
impl Policy for Out {}
impl Policy for InOut {}
impl Read for In {}
impl Write for Out {}
impl Read for InOut {}
impl Write for InOut {}

impl<T, P: Policy> Debug for UserPtr<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.ptr)
    }
}

// FIXME: Read nomicon to see why it's safe?
unsafe impl<T, P: Policy> Sync for UserPtr<T, P> {}
unsafe impl<T, P: Policy> Send for UserPtr<T, P> {}

/// 只读用户指针
pub type UserInPtr<T> = UserPtr<T, In>;
/// 只写用户指针
pub type UserOutPtr<T> = UserPtr<T, Out>;
/// 读写用户指针
pub type UserInOutPtr<T> = UserPtr<T, InOut>;

type Result<T> = core::result::Result<T, SysError>;

impl<T, P: Policy> UserPtr<T, P> {
    /// Get raw pointer
    pub fn ptr(&self) -> *mut T {
        self.ptr
    }

    /// Check if is null
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    /// Pointer add
    pub fn add(&self, count: usize) -> Self {
        UserPtr {
            ptr: unsafe { self.ptr.add(count) },
            mark: PhantomData,
        }
    }

    /// Check if is legal
    ///
    /// Return true if the pointer in not null and is aligned
    pub fn check(&self) -> Result<()> {
        if !self.is_null() && (self.ptr as usize) % core::mem::align_of::<T>() == 0 {
            Ok(())
        } else {
            Err(SysError::EFAULT)
        }
    }
}

impl<T, P: Read> UserPtr<T, P> {
    pub fn read(&self) -> Result<T> {
        if let Some(res) = copy_from_user(self.ptr) {
            Ok(res)
        } else {
            Err(SysError::EFAULT)
        }
    }
}

impl<T, P: Write> UserPtr<T, P> {
    /// Overwrites a memory location with the given `value`
    /// **without** reading or dropping the old value.
    pub fn write(&mut self, value: T) -> Result<()> {
        self.check()?;
        unsafe { self.ptr.write(value) };
        Ok(())
    }

    /// Same as [`write`](Self::write),
    /// but does nothing and returns [`Ok`] when pointer is null.
    pub fn write_if_not_null(&mut self, value: T) -> Result<()> {
        if !self.is_null() {
            self.write(value)
        } else {
            Ok(())
        }
    }

    /// Copies `values.len() * size_of<T>` bytes from `values` to `self`.
    /// The source and destination may not overlap.
    pub fn write_array(&mut self, values: &[T]) -> Result<()> {
        if !values.is_empty() {
            self.check()?;
            unsafe {
                self.ptr
                    .copy_from_nonoverlapping(values.as_ptr(), values.len());
            };
        }
        Ok(())
    }
}

impl<T, P: Policy> From<usize> for UserPtr<T, P> {
    fn from(value: usize) -> Self {
        UserPtr {
            ptr: value as _,
            mark: PhantomData,
        }
    }
}
