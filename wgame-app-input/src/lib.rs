//! Input event multiplexer for window events.
//!
//! Provides event distribution to multiple consumers with configurable capacity.

#![forbid(unsafe_code)]

use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    num::NonZero,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};

use futures::Stream;
pub use winit::{
    event::{self, WindowEvent as Event},
    keyboard,
};

/// Event multiplexer that distributes events to multiple input streams.
#[derive(Default)]
pub struct EventHandler {
    states: Vec<Weak<State>>,
    terminated: bool,
}

/// A stream of window events that can be polled asynchronously.
pub struct Input {
    state: Rc<State>,
}

struct State {
    capacity: Cell<Option<NonZero<usize>>>,
    terminated: Cell<bool>,
    events: RefCell<VecDeque<Event>>,
    waker: Cell<Waker>,
}

impl EventHandler {
    /// Push an event to all registered input streams.
    pub fn push(&mut self, event: Event) {
        if self.terminated {
            return;
        }
        self.states.retain_mut(|state| match state.upgrade() {
            Some(state) => {
                match &event {
                    Event::RedrawRequested => (),
                    _ => state.push_event(event.clone()),
                }
                true
            }
            None => false,
        });
    }

    /// Terminate all input streams by clearing all registered states.
    pub fn terminate(&mut self) {
        self.terminated = true;
        for state in self.states.drain(..).filter_map(|s| s.upgrade()) {
            state.terminated.set(true);
            state.waker.replace(Waker::noop().clone()).wake();
        }
    }

    /// Create a new input stream from this handler.
    pub fn input(&mut self) -> Input {
        self.states.retain(|state| state.strong_count() > 0);
        let state = Rc::new(State {
            terminated: Cell::new(self.terminated),
            ..Default::default()
        });
        self.states.push(Rc::downgrade(&state));
        Input { state }
    }
}

impl Input {
    /// Get the current event capacity for this input stream.
    pub fn capacity(&self) -> Option<NonZero<usize>> {
        self.state.capacity.get()
    }

    /// Set the event capacity for this input stream.
    pub fn set_capacity(&mut self, capacity: Option<NonZero<usize>>) {
        self.state.capacity.set(capacity);
    }

    /// Try to get the next event from the buffer without waiting.
    pub fn try_next(&mut self) -> Option<Event> {
        self.state.pop_event()
    }

    /// Check if this input stream has been terminated.
    pub fn is_terminated(&self) -> bool {
        self.state.terminated.get() && self.state.len() == 0
    }
}

impl Stream for Input {
    type Item = Event;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(event) = self.state.pop_event() {
            Poll::Ready(Some(event))
        } else if self.state.terminated.get() {
            Poll::Ready(None)
        } else {
            self.state.waker.set(cx.waker().clone());
            Poll::Pending
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.state.len(), None)
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            capacity: Cell::new(Self::DEFAULT_CAPACITY),
            terminated: Cell::new(false),
            events: RefCell::new(VecDeque::new()),
            waker: Cell::new(Waker::noop().clone()),
        }
    }
}

impl State {
    /// Default event capacity.
    pub const DEFAULT_CAPACITY: Option<NonZero<usize>> = Some(NonZero::new(1024).unwrap());

    fn push_event(&self, event: Event) {
        let mut events = self.events.borrow_mut();
        while let Some(cap) = self.capacity.get()
            && events.len() >= cap.get()
        {
            let old_event = events.pop_front().unwrap();
            log::warn!("Event overwritten due to overflow: {old_event:?}");
        }

        events.push_back(event);
        drop(events);
        self.waker.replace(Waker::noop().clone()).wake();
    }

    fn pop_event(&self) -> Option<Event> {
        self.events.borrow_mut().pop_front()
    }

    fn len(&self) -> usize {
        self.events.borrow().len()
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        self.terminate();
    }
}
impl futures::stream::FusedStream for Input {
    fn is_terminated(&self) -> bool {
        Input::is_terminated(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        task::Wake,
    };
    struct Counter(AtomicUsize);
    impl Wake for Counter {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    #[test]
    fn termination_and_drop_wake_waiters() {
        for explicit in [false, true] {
            let mut handler = EventHandler::default();
            let mut input = handler.input();
            let counter = Arc::new(Counter(AtomicUsize::new(0)));
            let waker = Waker::from(counter.clone());
            let mut cx = Context::from_waker(&waker);
            assert!(Pin::new(&mut input).poll_next(&mut cx).is_pending());
            if explicit {
                handler.terminate();
            } else {
                drop(handler);
            }
            assert_eq!(counter.0.load(Ordering::SeqCst), 1);
            assert!(matches!(
                Pin::new(&mut input).poll_next(&mut cx),
                Poll::Ready(None)
            ));
        }
    }
    #[test]
    fn overflow_keeps_latest_and_close_drains_buffer() {
        let mut handler = EventHandler::default();
        let mut input = handler.input();
        input.set_capacity(NonZero::new(1));
        handler.push(Event::Focused(false));
        handler.push(Event::Focused(true));
        handler.terminate();
        assert!(!input.is_terminated());
        assert_eq!(input.try_next(), Some(Event::Focused(true)));
        assert!(input.is_terminated());
        let mut closed = handler.input();
        handler.push(Event::Focused(false));
        assert!(closed.is_terminated());
        assert_eq!(closed.try_next(), None);
    }
}
