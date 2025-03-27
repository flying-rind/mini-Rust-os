//! 初始化内核配置
pub use super::imp::KernelConfig;
use crate::utils::InitOnce;

pub(crate) static KCONFIG: InitOnce<KernelConfig> = InitOnce::new();
