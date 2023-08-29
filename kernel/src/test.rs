//! 单元测试基础设施

use x86_64::instructions::port::Port;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
/// Qemu退出代码
pub enum QemuExitCode {
    /// 正常退出
    Success = 0x10,
    /// 异常退出
    Failed = 0x11,
}

/// 执行退出qemu操作
pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// 为test_runner的参数tests中的所有test外包一层输出
pub trait Testable {
    /// 执行test_case
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

/// 所有标记为#[test_case]的函数都会被加入tests队列中，本函数循环执行所有测试用例
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}
