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

    fn take_ready(&mut self) -> Vec<Waker> {
        let mut ready = Vec::new();
        let now = Instant::now();
        while let Some(peek) = self.queue.peek_mut() {
            if peek.0.waker.strong_count() == 0 {
                PeekMut::pop(peek);
                continue;
            }
            if peek.0.timestamp <= now {
                log::trace!("timer fired: {:?}", peek.0.timestamp);
                if let Some(waker) = PeekMut::pop(peek).0.waker.upgrade() {
                    ready.push(waker.borrow().clone());
                }
            } else {
                break;
            }
        }
        ready
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

    pub fn poll(&mut self) -> (ControlFlow, Vec<Waker>) {
        let ready = self.take_ready();
        // Web event loops can deliver the resulting user events after
        // AboutToWait, without another AboutToWait in that iteration. A further
        // poll is needed to run the tasks those events mark ready.
        let flow = if ready.is_empty() {
            self.schedule()
        } else {
            ControlFlow::Poll
        };
        (flow, ready)
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dropped_timers_do_not_schedule_spurious_wakes() {
        let mut queue = TimerQueue::default();
        let timer = queue.insert(Instant::now() + std::time::Duration::from_secs(60));
        assert!(matches!(queue.poll().0, ControlFlow::WaitUntil(_)));
        drop(timer);
        assert!(matches!(queue.poll().0, ControlFlow::Wait));
    }
    #[test]
    fn past_deadline_completes_on_first_poll() {
        let mut queue = TimerQueue::default();
        let mut timer = queue.insert(Instant::now());
        assert!(
            Pin::new(&mut timer)
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_ready()
        );
        drop(timer);
        assert!(matches!(queue.poll().0, ControlFlow::Wait));
    }

    #[test]
    fn expired_timer_requests_a_poll_before_returning_to_idle() {
        use std::{sync::Arc, task::Wake, time::Duration};

        struct Counter(std::sync::atomic::AtomicUsize);
        impl Wake for Counter {
            fn wake(self: Arc<Self>) {
                self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }
        let counter = Arc::new(Counter(Default::default()));
        let waker = Waker::from(counter.clone());
        let mut queue = TimerQueue::default();
        let deadline = Instant::now() + Duration::from_secs(60);
        let mut timer = queue.insert(deadline);
        assert!(
            Pin::new(&mut timer)
                .poll(&mut Context::from_waker(&waker))
                .is_pending()
        );
        assert!(matches!(queue.poll().0, ControlFlow::WaitUntil(_)));
        // Advance only the queue's deadline, so the test needs no wall-clock wait.
        queue.queue.peek_mut().unwrap().0.timestamp = Instant::now();
        let (flow, ready) = queue.poll();
        assert_eq!(flow, ControlFlow::Poll);
        assert_eq!(counter.0.load(std::sync::atomic::Ordering::SeqCst), 0);
        for waker in ready {
            waker.wake();
        }
        assert_eq!(counter.0.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(queue.poll().0, ControlFlow::Wait);
    }
}
