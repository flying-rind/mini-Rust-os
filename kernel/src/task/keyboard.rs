//! 键盘事件流

use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::stream::Stream;
use futures_util::stream::StreamExt;
use futures_util::task::AtomicWaker;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Once;

/// 键盘事件队列
static SCANCODE_QUEUE: Once<ArrayQueue<u8>> = Once::new();

/// 键盘协程唤醒器
static KEYBOARD_WAKER: AtomicWaker = AtomicWaker::new();

/// 添加键盘事件
pub fn add_scancode(scancode: u8) {
    if let Some(queue) = SCANCODE_QUEUE.get() {
        if let Err(_) = queue.push(scancode) {
            serial_println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            KEYBOARD_WAKER.wake();
        }
    } else {
        serial_println!("WARNING: scancode queue uninitialized");
    }
}

/// 异步事件流
pub struct ScancodeStream;

impl ScancodeStream {
    /// 创建键盘事件流
    pub fn new() -> Self {
        SCANCODE_QUEUE.call_once(|| ArrayQueue::new(100));
        ScancodeStream
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .get()
            .expect("scancode queue not initialized");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        KEYBOARD_WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scancode) => {
                KEYBOARD_WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

/// 异步任务
pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        // fb_print!("{}", character);
                        crate::shell::shell_input(character);
                    }
                    DecodedKey::RawKey(_key) => {
                        // fb_print!("{:?}", key);
                    }
                }
            }
        }
    }
}
