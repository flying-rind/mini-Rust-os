//! 内核对象的基对象

use super::*;
use alloc::string::String;
use core::fmt::Debug;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;
use downcast_rs::impl_downcast;
use downcast_rs::DowncastSync;
use rights::*;
use spin::Mutex;

mod handle;
mod rights;

/// 内核对象ID类型
pub type KoID = u64;
/// 内核对象的通用接口
pub trait KernelObject: DowncastSync + Debug {
    /// 获取内核对象KoID
    fn id(&self) -> KoID;
    /// 获取内核对象类型名称
    fn type_name(&self) -> &str;
    /// 获取内核对象的名称
    fn name(&self) -> &str;
    /// 设置内核对象名称
    fn set_name(&self, name: &str);
    // 信号相关.....
}

impl_downcast!(sync KernelObject);

/// 微内核中每个内核对象包含的基类
pub struct KObjectBase {
    /// KoID.
    pub id: KoID,
    inner: Mutex<KObjectBaseInner>,
}

/// `KObjectBase`的可变部分
#[derive(Default)]
pub struct KObjectBaseInner {
    name: String,
}

impl KObjectBase {
    /// 创建新内核基对象
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建新KoID
    pub fn new_koid() -> KoID {
        static KOID: AtomicU64 = AtomicU64::new(1024);
        KOID.fetch_add(1, Ordering::SeqCst)
    }

    /// 获得内核对象的名字
    pub fn name(&self) -> String {
        self.inner.lock().name.clone()
    }

    /// 设置内核对象的名字
    pub fn set_name(&self, name: &str) {
        self.inner.lock().name = String::from(name);
    }
}

impl Default for KObjectBase {
    fn default() -> Self {
        KObjectBase {
            id: Self::new_koid(),
            inner: Default::default(),
        }
    }
}

/// 使用这个宏来为内核对象实现内核对象通用接口，转发到内部的内核基对象
#[macro_export]
macro_rules! impl_kobject {
    ($class:ident $( $fn:tt)*) => {
        impl $crate::object::KernelObject for $class {
            // 获取内核对象KoID
            fn id(&self) -> KoID {
                self.base.id
            }
            // 获取内核对象类型名称
            fn type_name(&self) -> &str {
                stringify!($class)
            }
            // 获取内核对象的名称
            fn name(&self) -> &str {
                self.base.name()
            }
            // 设置内核对象名称
            fn set_name(&self, name: &str) {
                self.base.set_name(name)
            }
            // 信号相关.....
        }
    };
}
