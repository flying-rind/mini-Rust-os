//! Futex struct.

use alloc::boxed::Box;
use alloc::{collections::vec_deque::VecDeque, sync::Arc};
use core::task::Poll;
use core::{task::Waker, time::Duration};
use hal::time::timer_now;
use spin::Mutex;
use user_syscall::SysResult;

use crate::sync::NAIVE_TIMER;

pub struct Waiter {
    waker: Option<Waker>,
    woken: bool,
    futex: Arc<Futex>,
}

pub struct FutexInner {
    waiters: VecDeque<Arc<Mutex<Waiter>>>,
}

pub struct Futex {
    pub inner: Mutex<FutexInner>,
}

impl Futex {
    pub fn new() -> Self {
        Futex {
            inner: Mutex::new(FutexInner {
                waiters: VecDeque::new(),
            }),
        }
    }

    pub fn wake(&self, wake_count: usize) -> usize {
        let mut inner = self.inner.lock();
        for i in 0..wake_count {
            if let Some(waiter) = inner.waiters.pop_front() {
                let mut waiter = waiter.lock();
                waiter.woken = true;
                if let Some(waker) = waiter.waker.take() {
                    waker.wake();
                } else {
                    return i;
                }
            }
        }
        wake_count
    }

    pub fn wait(self: &Arc<Self>, timeout: Option<Duration>) -> impl Future<Output = SysResult> {
        struct FutexFuture {
            waiter: Arc<Mutex<Waiter>>,
            deadline: Option<Duration>,
        }

        impl Future for FutexFuture {
            type Output = SysResult;

            fn poll(
                self: core::pin::Pin<&mut Self>,
                cx: &mut core::task::Context<'_>,
            ) -> core::task::Poll<Self::Output> {
                let mut waiter = self.waiter.lock();
                if waiter.woken {
                    return Poll::Ready(Ok(0));
                }
                if let Some(deadline) = self.deadline {
                    if timer_now() >= deadline {
                        waiter.woken = true;
                        return Poll::Ready(Err(hal::SysError::ETIMEDOUT));
                    }
                }
                // Polled first time?
                if waiter.waker.is_none() {
                    let mut futex = waiter.futex.inner.lock();
                    futex.waiters.push_back(self.waiter.clone());
                    drop(futex);
                    waiter.waker.replace(cx.waker().clone());

                    if let Some(deadline) = self.deadline {
                        let waker = cx.waker().clone();
                        NAIVE_TIMER
                            .lock()
                            .add(deadline, Box::new(move |_| waker.wake()));
                    }
                }
                Poll::Pending
            }
        }
        FutexFuture {
            waiter: Arc::new(Mutex::new(Waiter {
                waker: None,
                woken: false,
                futex: self.clone(),
            })),
            deadline: timeout.map(|t| timer_now() + t),
        }
    }
}
