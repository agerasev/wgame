use std::{
    cell::RefCell,
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, binary_heap::PeekMut},
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};

use futures::future::FusedFuture;
use winit::event_loop::ControlFlow;

#[cfg(feature = "std")]
pub use std::time::Instant;
#[cfg(feature = "web")]
pub use web_time::Instant;

pub struct Timer {
    timestamp: Instant,
    waker: Rc<RefCell<Waker>>,
}

impl Timer {
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }
}

struct InnerTimer {
    timestamp: Instant,
    waker: Weak<RefCell<Waker>>,
}

impl PartialEq for InnerTimer {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.timestamp.eq(&other.timestamp) && Weak::ptr_eq(&self.waker, &other.waker)
    }
}

impl Eq for InnerTimer {}

impl PartialOrd for InnerTimer {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InnerTimer {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .then(self.waker.as_ptr().cmp(&other.waker.as_ptr()))
    }
}

#[derive(Default)]
pub(crate) struct TimerQueue {
    queue: BinaryHeap<Reverse<InnerTimer>>,
}

impl TimerQueue {
    pub fn insert(&mut self, timestamp: Instant) -> Timer {
        let timer = Timer {
            timestamp,
            waker: Rc::new(RefCell::new(Waker::noop().clone())),
        };
        let inner_timer = InnerTimer {
            timestamp,
            waker: Rc::downgrade(&timer.waker),
        };

        let timestamp = timer.timestamp;
        self.queue.push(Reverse(inner_timer));
        log::trace!("timer added: {timestamp:?}");

        timer
    }

    fn wake(&mut self) {
        let now = Instant::now();
        while let Some(peek) = self.queue.peek_mut() {
            if peek.0.timestamp <= now {
                log::trace!("timer fired: {:?}", peek.0.timestamp);
                if let Some(waker) = PeekMut::pop(peek).0.waker.upgrade() {
                    waker.borrow().wake_by_ref();
                }
            } else {
                break;
            }
        }
    }

    fn schedule(&self) -> ControlFlow {
        if let Some(Reverse(timer)) = self.queue.peek() {
            log::trace!("waiting until: {:?}", timer.timestamp);
            ControlFlow::WaitUntil(timer.timestamp)
        } else {
            log::trace!("waiting indefinitely");
            ControlFlow::Wait
        }
    }

    pub fn poll(&mut self) -> ControlFlow {
        self.wake();
        self.schedule()
    }
}

impl Future for Timer {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.is_terminated() {
            Poll::Ready(())
        } else {
            *self.waker.borrow_mut() = cx.waker().clone();
            Poll::Pending
        }
    }
}

impl FusedFuture for Timer {
    fn is_terminated(&self) -> bool {
        Instant::now() >= self.timestamp
    }
}
