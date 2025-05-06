//! 物理页帧
use super::{frame_allocator::*, phys_to_virt, PAGE_SIZE};

/// 物理页帧
#[repr(transparent)]
pub struct PhysFrame(pub usize);

impl PhysFrame {
    /// 分配一个物理页帧
    pub fn alloc() -> Option<Self> {
        allocate_frame().map(Self)
    }

    /// 获取页帧起始物理地址
    pub fn start_paddr(&self) -> usize {
        self.0
    }
    /// 分配一个物理页帧且清零
    pub fn alloc_zero() -> Option<Self> {
        let frame = allocate_frame().map(Self).unwrap();
        clear_frames(frame.0, 1);
        Some(frame)
    }

    /// 解析为切片
    pub fn as_slice(&self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(phys_to_virt(self.0) as *mut u8, PAGE_SIZE) }
    }
}

impl Drop for PhysFrame {
    /// drop时释放掉物理页帧占的位
    fn drop(&mut self) {
        deallocate_frame(self.0);
    }
}
