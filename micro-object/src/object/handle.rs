//! 用户态持有的内核对象的句柄
use {super::*, alloc::sync::Arc};

/// 用户态表示Handle的数值
pub type HandleValue = u32;

/// 无效的handle值
pub const INVALID_HANDLE: HandleValue = 0;

/// A Handle is how a specific process refers to a specific kernel object.
#[derive(Debug, Clone)]
pub struct Handle {
    /// 内核对象的引用
    pub object: Arc<dyn KernelObject>,
    /// 与Handle关联的权限
    pub rights: Rights,
}
