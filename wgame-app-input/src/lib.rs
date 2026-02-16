//! Input handling utilities for wgame applications.
//!
//! This crate provides an event multiplexer system that allows multiple consumers
//! to receive and process window events from winit. It implements the `Stream`
//! trait from `futures`, making it easy to use with async/await code.
//!
//! # Overview
//!
//! The crate provides two main types:
//! - [`EventHandler`] - The central event dispatcher that receives events and
//!   distributes them to all registered inputs
//! - [`Input`] - A stream of events that can be polled to receive new events
//!
//! # Event Multiplexing
//!
//! Multiple `Input` streams can be created from a single `EventHandler`. When
//! an event is pushed to the handler, it is distributed to all active inputs.
//! This allows different parts of your application to receive the same events.
//!
//! # Event Capacity
//!
//! Each `Input` has a configurable event capacity. When the event buffer is
//! full, new events will overwrite the oldest events. This prevents memory
//! exhaustion when events arrive faster than they are processed.
//!
//! # Examples
//!
//! Creating and using an input stream:
//! ```
//! use futures::StreamExt;
//! use wgame_app_input::{EventHandler, Event};
//!
//! # async fn example() {
//! let mut handler = EventHandler::default();
//! let mut input = handler.input();
//!
//! // Push some events
//! handler.push(Event::RedrawRequested);
//! handler.push(Event::WindowCloseRequested);
//!
//! // Poll events from the input stream
//! while let Some(event) = input.try_next() {
//!     println!("Received event: {:?}", event);
//! }
//! # }
//! ```
//!
//! Using input as a stream in async code:
//! ```
//! use futures::StreamExt;
//! use wgame_app_input::{EventHandler, Event};
//!
//! # async fn example() {
//! let mut handler = EventHandler::default();
//! let mut input = handler.input();
//!
//! // This will wait until an event is available
//! if let Some(event) = input.next().await {
//!     println!("Received event: {:?}", event);
//! }
//! # }
//! ```
//!
//! Setting event capacity:
//! ```
//! use std::num::NonZero;
//! use wgame_app_input::EventHandler;
//!
//! let mut handler = EventHandler::default();
//! let mut input = handler.input();
//!
//! // Set capacity to 100 events
//! input.set_capacity(Some(NonZero::new(100).unwrap()));
//! ```

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
///
/// The `EventHandler` acts as a central hub for event distribution. When events
/// are pushed to it, they are forwarded to all active `Input` streams that were
/// created from this handler.
///
/// # Examples
///
/// ```
/// use wgame_app_input::EventHandler;
///
/// let mut handler = EventHandler::default();
///
/// // Create multiple inputs from the same handler
/// let input1 = handler.input();
/// let input2 = handler.input();
///
/// // Events pushed here will be available on both inputs
/// handler.push(winit::event::Event::WindowCloseRequested);
/// ```
#[derive(Default)]
pub struct EventHandler {
    states: Vec<Weak<State>>,
}

/// A stream of window events that can be polled asynchronously.
///
/// The `Input` type implements `futures::Stream`, allowing it to be used with
/// async/await code. Multiple `Input` instances can be created from the same
/// `EventHandler`, and they will all receive the same events.
///
/// # Examples
///
/// ```
/// use futures::StreamExt;
/// use wgame_app_input::EventHandler;
///
/// # async fn example() {
/// let mut handler = EventHandler::default();
/// let mut input = handler.input();
///
/// // Wait for the next event
/// if let Some(event) = input.next().await {
///     println!("Event: {:?}", event);
/// }
/// # }
/// ```
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
    ///
    /// The event will be distributed to all active `Input` instances. Events
    /// that are already redraw requests are not cloned and distributed (they
    /// are handled specially).
    ///
    /// # Arguments
    /// * `event` - The event to push to all inputs
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
    ///
    /// After calling this method, all `Input` instances will be terminated
    /// and will return `None` when polled.
    pub fn terminate(&mut self) {
        self.states.clear();
    }

    /// Create a new input stream from this handler.
    ///
    /// The returned `Input` will receive all events pushed to this handler
    /// until it is dropped or the handler is terminated.
    pub fn input(&mut self) -> Input {
        self.states.retain(|state| state.strong_count() > 0);
        let state = Rc::new(State::default());
        self.states.push(Rc::downgrade(&state));
        Input { state }
    }
}

impl Input {
    /// Get the current event capacity for this input stream.
    ///
    /// When the event buffer is full, new events will overwrite the oldest
    /// events. Returns `None` if there is no capacity limit.
    pub fn capacity(&self) -> Option<NonZero<usize>> {
        self.state.capacity.get()
    }

    /// Set the event capacity for this input stream.
    ///
    /// When the event buffer reaches this capacity, new events will overwrite
    /// the oldest events. Set to `None` to disable capacity limiting.
    ///
    /// # Arguments
    /// * `capacity` - The maximum number of events to buffer, or `None` for unlimited
    pub fn set_capacity(&mut self, capacity: Option<NonZero<usize>>) {
        self.state.capacity.set(capacity);
    }

    /// Try to get the next event from the buffer without waiting.
    ///
    /// Returns `Some(event)` if an event is available, or `None` if the buffer
    /// is empty.
    pub fn try_next(&mut self) -> Option<Event> {
        self.state.pop_event()
    }

    /// Check if this input stream has been terminated.
    ///
    /// An input is terminated when the `EventHandler` is dropped or when
    /// `EventHandler::terminate` is called.
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
