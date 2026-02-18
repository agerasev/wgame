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
}

/// A stream of window events that can be polled asynchronously.
pub struct Input {
    state: Rc<State>,
}

struct State {
    capacity: Cell<Option<NonZero<usize>>>,
    events: RefCell<VecDeque<Event>>,
    waker: Cell<Waker>,
}

impl EventHandler {
    /// Push an event to all registered input streams.
    pub fn push(&mut self, event: Event) {
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
        self.states.clear();
    }

    /// Create a new input stream from this handler.
    pub fn input(&mut self) -> Input {
        self.states.retain(|state| state.strong_count() > 0);
        let state = Rc::new(State::default());
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
        Rc::weak_count(&self.state) == 0
    }
}

impl Stream for Input {
    type Item = Event;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.is_terminated() {
            self.state.poll_next_event(cx).map(Some)
        } else {
            Poll::Ready(None)
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
            events: RefCell::new(VecDeque::new()),
            waker: Cell::new(Waker::noop().clone()),
        }
    }
}

impl State {
    /// Default event capacity (1024 events).
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

        self.waker.replace(Waker::noop().clone()).wake();
    }

    fn pop_event(&self) -> Option<Event> {
        self.events.borrow_mut().pop_front()
    }

    fn poll_next_event(&self, cx: &mut Context<'_>) -> Poll<Event> {
        if let Some(event) = self.events.borrow_mut().pop_front() {
            Poll::Ready(event)
        } else {
            self.waker.set(cx.waker().clone());
            Poll::Pending
        }
    }

    fn len(&self) -> usize {
        self.events.borrow().len()
    }
}
