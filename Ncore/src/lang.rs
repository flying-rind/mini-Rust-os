pub use hybrid_objects::task::*;
use core::panic::PanicInfo;
use hybrid_objects::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // 打印错误信息
    if let Some(l) = info.location() {
        println!(
            "\x1b[31m[kernel] Panicked at {}:{} {}\x1b[0m",
            l.file(),
            l.line(),
            info.message()
        );
    } else {
        println!("[Kernel] Panicked: {}", info.message());
    }
    // 若是内核服务线程崩溃了，尝试恢复错误
    let current_kthread = CURRENT_KTHREAD.get().as_ref().unwrap().clone();
    match current_kthread.ktype() {
        KthreadType::ROOT | KthreadType::EXECUTOR | KthreadType::UNKNOWN => {
            println!("[Panic handler] Cannot reboot!");
        }
        KthreadType::BLK | KthreadType::FS => {
            let current_req_id = current_kthread.current_request_id();
            println!(
                "\x1b[31m[Panic handler] Trying to Reboot..., the dangerous request(ID: {}) will be dropped!\x1b[0m",
                current_req_id
            );
            // 重启内核线程
            current_kthread.reboot(current_kthread.clone());
        }
    }
    loop {}
}
