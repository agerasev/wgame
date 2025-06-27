use std::{
    cell::Cell,
    cmp::Reverse,
    collections::{BinaryHeap, binary_heap::PeekMut},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
    time::Instant,
};

use winit::event_loop::ControlFlow;

#[derive(Clone)]
pub struct Timer {
    timestamp: Instant,
    waker: Rc<Cell<Option<Waker>>>,
}

impl Timer {
    pub fn timestamp(&self) -> &Instant {
        &self.timestamp
    }
}

impl PartialEq for Timer {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.timestamp.eq(&other.timestamp)
    }
}

impl Eq for Timer {}

impl PartialOrd for Timer {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timer {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

#[derive(Default)]
pub struct TimerQueue {
    queue: BinaryHeap<Reverse<Timer>>,
}

impl TimerQueue {
    pub fn add(&mut self, timestamp: Instant) -> Timer {
        let timer = Timer {
            timestamp,
            waker: Default::default(),
        };

        let timestamp = timer.timestamp;
        self.queue.push(Reverse(timer.clone()));
        log::trace!("timer added: {:?}", timestamp);

        timer
    }

    fn wake(&mut self) {
        let now = Instant::now();
        while let Some(peek) = self.queue.peek_mut() {
            if peek.0.timestamp <= now {
                log::trace!("timer fired: {:?}", peek.0.timestamp);
                if let Some(waker) = peek.0.waker.take() {
                    waker.wake();
                }
                PeekMut::pop(peek);
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
        if Instant::now() >= self.timestamp {
            Poll::Ready(())
        } else {
            self.waker.set(Some(cx.waker().clone()));
            Poll::Pending
        }
    }
}
