//! 中断控制器模块

use pic8259::ChainedPics;
use spin::Mutex;

/// 一级PIC偏移
pub const PIC_1_OFFSET: u8 = 32;

/// 二级PIC偏移
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
/// 外设中断编号
pub enum InterruptIndex {
    /// 时钟
    Timer = PIC_1_OFFSET,
    /// 键盘
    Keyboard,
    /// 级联PIC芯片
    Cascade,
    /// 串口2
    Com2,
    /// 串口1
    Com1,
    /// Parallel Port 2/3
    Lpt2,
    /// Floppy disk
    Floppy,
    /// Parallel Port 1
    Lpt1,
    /// Real Time Clock
    RTC = PIC_2_OFFSET,
    /// ACPI
    ACPI,
    /// Available
    Avail1,
    /// Available
    Avail2,
    /// 鼠标
    Mouse,
    /// Co-Processor
    FPU,
    /// Primary ATA
    ATA1,
    /// Secondary ATA
    ATA2,
}

/// 级联PIC驱动全局变量
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// 初始化中断控制器
pub fn init() {
    unsafe { PICS.lock().initialize() };

    // 打开串口中断
    // let mut masks = unsafe { PICS.lock().read_masks() };
    // BIOS模式启动默认掩码[0xb8, 0x8e]，UEFI模式启动默认掩码[0xff, 0xff]
    // 需要开启时钟（第0位）、键盘（第1位）、串口1中断（第4位）
    // masks[0] = masks[0] & 0b1110_1100;

    // 在BIOS的基础上打开串口应该是[0xa8, 0x8e]
    let masks = [0xa8, 0x8e];
    unsafe { PICS.lock().write_masks(masks[0], masks[1]) };

    // CPU端开启中断
    // x86_64::instructions::interrupts::enable();
}

/// 通知完成中断
pub fn notify_eoi(irq: u8) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(irq);
    }
}
