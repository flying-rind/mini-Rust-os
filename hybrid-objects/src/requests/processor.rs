//! 请求响应器
use super::*;

/// 解析请求类型并进行具体地处理
pub trait Processor: Send + Sync {
    /// 处理当前请求，完毕后唤醒相应的等待协程
    fn process_request(&self, request: Request);
}
