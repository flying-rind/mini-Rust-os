//! 设备驱动
//! 


/// 初始化串口
pub(super) fn init_early(){
    super::super::common::console::init(0x3f8);   
}