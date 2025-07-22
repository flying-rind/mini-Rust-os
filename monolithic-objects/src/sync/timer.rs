//! Simple Timer.

use alloc::{boxed::Box, collections::btree_map::BTreeMap};
use core::time::Duration;
use hal::time::timer_now;
use lazy_static::lazy_static;
use spin::Mutex;

/// The type of callback functions.
type Callback = Box<dyn FnOnce(Duration) + Send + Sync + 'static>;

lazy_static! {
    pub static ref NAIVE_TIMER: Mutex<Timer> = Mutex::new(Timer::default());
}

/// A simple timer.
#[derive(Default)]
pub struct Timer {
    /// Events registered in timer.
    events: BTreeMap<Duration, Callback>,
}

impl Timer {
    /// Add a timer.
    ///
    /// The `callback` will be called on timer expired after `deadline`.
    pub fn add(&mut self, mut deadline: Duration, callback: Callback) {
        while self.events.contains_key(&deadline) {
            deadline = deadline + Duration::from_nanos(1);
        }
        self.events.insert(deadline, Box::new(callback));
    }

    /// Expire timers.
    ///
    /// Given the current time `now`, trigger and remove all expired timers.
    pub fn expire(&mut self, now: Duration) {
        while let Some(entry) = self.events.first_entry() {
            if *entry.key() > now {
                return;
            }
            let (_, callback) = entry.remove_entry();
            callback(now);
        }
    }
}

/// Do timer.
pub fn timer() {
    // dotick()?

    let now = timer_now();
    NAIVE_TIMER.lock().expire(now);
}
