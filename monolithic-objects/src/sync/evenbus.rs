//! Eventbus
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use bitflags::bitflags;
use core::task::Poll;
use spin::Mutex;

bitflags! {
    #[derive(Default, Copy, Clone, PartialEq)]
    pub struct Event: u32 {
        /// File
        const READABLE                      = 1 << 0;
        const WRITABLE                      = 1 << 1;
        const ERROR                         = 1 << 2;
        const CLOSED                        = 1 << 3;

        /// Process
        const PROCESS_QUIT                  = 1 << 10;
        const CHILD_PROCESS_QUIT            = 1 << 11;
        const RECEIVE_SIGNAL                = 1 << 12;

        /// Semaphore
        const SEMAPHORE_REMOVED             = 1 << 20;
        const SEMAPHORE_CAN_ACQUIRE         = 1 << 21;
    }
}

/// Callback when event happens, return true when the waiting event happens.
pub type EventHandler = Box<dyn Fn(Event) -> bool + Send>;

#[derive(Default)]
pub struct EventBus {
    /// Waiting events
    event: Event,
    /// events callbacks
    callbacks: Vec<EventHandler>,
}

impl EventBus {
    /// Create a new eventbus.
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }

    /// Register a new event.
    pub fn set(&mut self, set: Event) {
        self.change(Event::empty(), set);
    }

    /// Unregister an evnet.
    pub fn clear(&mut self, reset: Event) {
        self.change(reset, Event::empty());
    }

    /// Change waiting events.
    pub fn change(&mut self, reset: Event, set: Event) {
        let orig = self.event;
        let mut new = self.event;
        new.insert(set);
        new.remove(reset);
        self.event = new;
        if new != orig {
            self.callbacks.retain(|f| !f(new));
        }
    }

    /// Add a callback
    pub fn subscribe(&mut self, callback: EventHandler) {
        self.callbacks.push(callback);
    }
}

/// Create a WaitForEvent Future.
pub fn wait_for_event(bus: Arc<Mutex<EventBus>>, event: Event) -> impl Future<Output = ()> {
    WaitForEvent { bus, event }
}

/// WaitForEvent Future
struct WaitForEvent {
    /// used bus
    bus: Arc<Mutex<EventBus>>,
    /// waiting event
    event: Event,
}

impl Future for WaitForEvent {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let mut lock = self.bus.lock();
        // Already happened
        if !(lock.event & self.event).is_empty() {
            return Poll::Ready(());
        }
        // Havn't happened yet.
        let waker = cx.waker().clone();
        let event = self.event;
        lock.subscribe(Box::new(move |s| {
            if (s & event).is_empty() {
                return false;
            }
            waker.wake_by_ref();
            true
        }));
        Poll::Pending
    }
}
