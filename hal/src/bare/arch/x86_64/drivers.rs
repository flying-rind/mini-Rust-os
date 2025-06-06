//! 设备驱动
//! 


/// 初始化串口
pub(super) fn drivers_init(){
    super::super::common::console::init(0x3f8);   
}