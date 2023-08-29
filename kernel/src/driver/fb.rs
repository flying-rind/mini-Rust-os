//! 格式化输出至framebuffer，部分参考bootloader实现

use bootloader_api::info::FrameBuffer;
use core::cmp::max;
use core::fmt::Write;
use core::sync::atomic::Ordering;
use core::{cmp::min, fmt};
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};
use spin::{Mutex, Once};
use x86_64::instructions::interrupts;

use super::mouse::START_MOVE;

/// FrameBuffer驱动全局变量
pub static FRAME_BUFFER: Once<Mutex<FrameBufferDriver>> = Once::new();

/// 初始化FrameBuffer驱动
pub fn init(frame_buffer: &'static mut FrameBuffer) {
    serial_println!("[Kernel] {:#x?}", frame_buffer);

    // 根据BootInfo传入信息创建FrameBuffer驱动
    FRAME_BUFFER.call_once(|| Mutex::new(FrameBufferDriver::new(frame_buffer)));
}

/// 辅助打印结构，主要实现Write trait
struct Printer;
impl Write for Printer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        interrupts::without_interrupts(|| {
            FRAME_BUFFER
                .get()
                .and_then(|fb| Some(fb.lock().write_str(s)))
                .expect("Uninit frame buffer");
        });
        Ok(())
    }
}

/// 打印至FrameBuffer
pub fn print(args: fmt::Arguments) {
    // write_fmt函数在Write trait中定义，因此需要实现Write trait
    Printer.write_fmt(args).unwrap();
}

/// 格式化打印至FrameBuffer（无换行）
#[macro_export]
macro_rules! fb_print {
    ($($arg:tt)*) => {
        $crate::driver::fb::print(format_args!($($arg)*))
    };
}

/// 格式化打印至FrameBuffer（有换行）
#[macro_export]
macro_rules! fb_println {
    () => ($crate::fb_print!("\n"));
    ($fmt:expr) => ($crate::fb_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::fb_print!(
        concat!($fmt, "\n"), $($arg)*));
}

// 以下代码参考bootloader实现

/// 行间距
const LINE_SPACING: usize = 2;

/// 字间距
const LETTER_SPACING: usize = 0;

/// 页边距
const BORDER_PADDING: usize = 1;

/// 字高
const LETTER_HEIGHT: RasterHeight = RasterHeight::Size16;

/// 字宽
const LETTER_WIDTH: usize = get_raster_width(FontWeight::Regular, LETTER_HEIGHT);

/// 字体
const FONT_WEIGHT: FontWeight = FontWeight::Regular;

/// 鼠标光标宽度
const MOUSE_WEIGHT: usize = 9;

/// 鼠标光标高度
const MOUSE_HEIGHT: usize = 16;

/// FrameBuffer驱动
pub struct FrameBufferDriver {
    buffer: &'static mut [u8],
    bytes_per_pixel: usize,
    height: usize,
    width: usize,
    cur_x_pos: usize,
    cur_y_pos: usize,
    mouse_x: usize,
    mouse_y: usize,
}

impl FrameBufferDriver {
    /// 创建FrameBuffer驱动
    pub fn new(framebuffer: &'static mut FrameBuffer) -> Self {
        let info = framebuffer.info();
        let buffer_start = framebuffer.buffer().as_ptr() as usize;
        let buffer_len = info.byte_len;

        let mut fb = Self {
            buffer: unsafe { core::slice::from_raw_parts_mut(buffer_start as *mut u8, buffer_len) },
            bytes_per_pixel: info.bytes_per_pixel,
            height: info.height,
            width: info.width,
            cur_x_pos: 0,
            cur_y_pos: 0,
            mouse_x: info.width / 2,
            mouse_y: info.height / 2,
        };
        fb.clear();
        fb
    }

    // 换行
    fn newline(&mut self) {
        self.cur_x_pos = BORDER_PADDING;
        self.cur_y_pos += LETTER_HEIGHT.val() + LINE_SPACING;
    }

    /// 清屏
    pub fn clear(&mut self) {
        self.cur_x_pos = BORDER_PADDING;
        self.cur_y_pos = BORDER_PADDING;
        self.buffer.fill(0);
    }

    /// 写字符串
    fn write_str(&mut self, s: &str) {
        for byte in s.chars() {
            self.write_char(byte);
        }
    }

    /// 写字符
    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\t' => self.write_str("    "),
            c => {
                // 分别判断x、y坐标是否越界
                let new_xpos = self.cur_x_pos + LETTER_WIDTH;
                if new_xpos >= self.width {
                    self.newline();
                }
                let new_ypos = self.cur_y_pos + LETTER_HEIGHT.val() + BORDER_PADDING;
                if new_ypos >= self.height {
                    self.clear();
                }
                // 渲染字符
                self.render_char(c);
            }
        }
    }

    /// 退格
    pub fn delete(&mut self) {
        self.cur_x_pos -= LETTER_WIDTH;
    }

    /// 渲染字符
    fn render_char(&mut self, c: char) {
        let rendered_char =
            get_raster(c, FONT_WEIGHT, LETTER_HEIGHT).expect("Failed to render a char");
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.write_pixel(self.cur_x_pos + x, self.cur_y_pos + y, *byte);
            }
        }
        self.cur_x_pos += rendered_char.width() + LETTER_SPACING;
    }

    /// 写像素
    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * self.width + x;
        let color = [intensity / 2, intensity, intensity, 0];
        let bytes_per_pixel = self.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.buffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
    }

    /// 逆或像素
    fn xor_pixel(&mut self, x: usize, y: usize) {
        let pixel_offset = y * self.width + x;
        let bytes_per_pixel = self.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        let blue = self.buffer[byte_offset];
        let green = self.buffer[byte_offset + 1];
        let red = self.buffer[byte_offset + 2];
        let color = [!blue, !green, !red, 0];
        self.buffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
    }

    /// 渲染鼠标
    pub fn mouse_render(&mut self, intensity: u8) {
        let (x, y) = (
            self.mouse_x - MOUSE_WEIGHT / 2,
            self.mouse_y - MOUSE_HEIGHT / 2,
        );
        for i in 0..MOUSE_WEIGHT {
            for j in 0..MOUSE_HEIGHT {
                self.write_pixel(x + i, y + j, intensity);
            }
        }
    }

    /// 逆或鼠标
    pub fn mouse_xor(&mut self) {
        let (x, y) = (
            self.mouse_x - MOUSE_WEIGHT / 2,
            self.mouse_y - MOUSE_HEIGHT / 2,
        );
        for i in 0..MOUSE_WEIGHT {
            for j in 0..MOUSE_HEIGHT {
                self.xor_pixel(x + i, y + j);
            }
        }
    }

    /// 鼠标移动
    pub fn mouse_move(&mut self, x: isize, y: isize) {
        if START_MOVE.load(Ordering::Relaxed) == true {
            self.mouse_xor();
        }
        let new_x = self.mouse_x as isize + x;
        let new_y = self.mouse_y as isize + y;
        self.mouse_x = min(
            max(new_x, MOUSE_WEIGHT as isize / 2) as usize,
            self.width - (MOUSE_WEIGHT + 1) / 2 - 1,
        );
        self.mouse_y = min(
            max(new_y, MOUSE_HEIGHT as isize / 2) as usize,
            self.height - (MOUSE_HEIGHT + 1) / 2 - 1,
        );
        self.mouse_xor();
    }

    /// 鼠标绘图
    pub fn mouse_print(&mut self, x: isize, y: isize) {
        let new_x = self.mouse_x as isize + x;
        let new_y = self.mouse_y as isize + y;
        self.mouse_x = min(
            max(new_x, MOUSE_WEIGHT as isize / 2) as usize,
            self.width - (MOUSE_WEIGHT + 1) / 2 - 1,
        );
        self.mouse_y = min(
            max(new_y, MOUSE_HEIGHT as isize / 2) as usize,
            self.height - (MOUSE_HEIGHT + 1) / 2 - 1,
        );
        self.mouse_render(255);
    }

    /// 鼠标擦除
    pub fn mouse_remove(&mut self, x: isize, y: isize) {
        let new_x = self.mouse_x as isize + x;
        let new_y = self.mouse_y as isize + y;
        self.mouse_x = min(
            max(new_x, MOUSE_WEIGHT as isize / 2) as usize,
            self.width - (MOUSE_WEIGHT + 1) / 2 - 1,
        );
        self.mouse_y = min(
            max(new_y, MOUSE_HEIGHT as isize / 2) as usize,
            self.height - (MOUSE_HEIGHT + 1) / 2 - 1,
        );
        self.mouse_render(0);
    }
}

/// 鼠标移动
pub fn mouse_move(x: isize, y: isize) {
    interrupts::without_interrupts(|| {
        FRAME_BUFFER.get().unwrap().lock().mouse_move(x, y);
    });
}

/// 鼠标绘图
pub fn mouse_print(x: isize, y: isize) {
    interrupts::without_interrupts(|| {
        FRAME_BUFFER.get().unwrap().lock().mouse_print(x, y);
    });
}

/// 鼠标擦除
pub fn mouse_remove(x: isize, y: isize) {
    interrupts::without_interrupts(|| {
        FRAME_BUFFER.get().unwrap().lock().mouse_remove(x, y);
    });
}

/// 清屏
pub fn clear() {
    interrupts::without_interrupts(|| {
        FRAME_BUFFER.get().unwrap().lock().clear();
    });
}

/// 退格
pub fn delete() {
    interrupts::without_interrupts(|| {
        FRAME_BUFFER.get().unwrap().lock().delete();
    });
}
