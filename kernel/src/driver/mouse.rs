//! 鼠标驱动

use core::sync::atomic::{AtomicBool, Ordering};
use ps2_mouse::{Mouse, MouseState};
use spin::{Mutex, Once};

use crate::driver::fb::{mouse_move, mouse_print, mouse_remove};

/// 鼠标设备驱动
pub static MOUSE: Once<Mutex<Mouse>> = Once::new();

/// 鼠标刚开始移动时不反转颜色
pub static START_MOVE: AtomicBool = AtomicBool::new(false);

/// 初始化鼠标设备
pub fn init() {
    let mouse = MOUSE.call_once(|| Mutex::new(Mouse::new()));
    mouse.lock().init().unwrap();
    mouse.lock().set_on_complete(on_complete);
}

/// 捕获鼠标事件
pub fn mouse_event(package: u8) {
    MOUSE.get().unwrap().lock().process_packet(package);
}

/// 处理鼠标事件
fn on_complete(mouse_state: MouseState) {
    let (x, y) = (mouse_state.get_x(), mouse_state.get_y());
    if mouse_state.left_button_down() {
        mouse_print(x as _, -y as _);
        START_MOVE.store(false, Ordering::Relaxed);
    }
    if mouse_state.right_button_down() {
        mouse_remove(x as _, -y as _);
        START_MOVE.store(false, Ordering::Relaxed);
    }
    if mouse_state.left_button_up() && mouse_state.right_button_up() {
        mouse_move(x as _, -y as _);
        START_MOVE.store(true, Ordering::Relaxed);
    }
}
