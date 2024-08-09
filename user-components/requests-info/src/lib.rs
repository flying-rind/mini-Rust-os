//! 定义用户态向内核线程发送的请求描述信息

#![no_std]
#![feature(linkage)]

extern crate alloc;

pub mod blkreqinfo;
pub mod fsreqinfo;

/// 用户请求信息和字节切片相互转化
pub trait CastBytes {
    /// 将请求转化为字节数组。
    /// 请求在发送给组件对象时，会被转化为字节数组。在实际处理时，才会转化回具体的请求类型
    fn as_bytes(&self) -> &[u8]
    where
        Self: Sized,
    {
        let data = self as *const Self as *const u8;
        let len = core::mem::size_of_val(self);
        unsafe { core::slice::from_raw_parts(data, len) }
    }

    /// 将字节数组转化为请求
    fn from_bytes(bytes: &[u8]) -> &Self
    where
        Self: Sized,
    {
        assert_eq!(bytes.len(), core::mem::size_of::<Self>());
        let ptr = bytes.as_ptr() as *const Self;
        unsafe { ptr.as_ref().unwrap() }
    }
}
