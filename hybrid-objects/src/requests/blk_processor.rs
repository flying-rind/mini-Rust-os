//! 块设备请求处理器

use super::*;

/// 块设备请求处理器
pub struct BlkProcessor;

impl Processor for BlkProcessor {
    /// 处理一个块设备请求
    #[allow(unused)]
    fn process_request(&self, request: Request) {
        unimplemented!()
    }
}
