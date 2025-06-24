use std::{
    cell::RefCell,
    collections::VecDeque,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use futures::Stream;
use winit::{
    error::OsError,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window as RawWindow, WindowAttributes},
};

use crate::{
    executor::TaskId,
    proxy::{AppProxy, SharedCallState},
};

pub struct WindowState {
    pub waker: Option<Waker>,
    pub events: VecDeque<WindowEvent>,
    pub events_capacity: usize,
    pub terminated: bool,
}

const EVENT_BUFFER_MIN_CAPACITY: usize = 0x10;

impl WindowState {
    fn new(events_capacity: Option<usize>) -> Self {
        Self {
            waker: None,
            events: VecDeque::new(),
            events_capacity: events_capacity
                .unwrap_or(usize::MAX)
                .max(EVENT_BUFFER_MIN_CAPACITY),
            terminated: false,
        }
    }

    pub fn push_event(&mut self, event: WindowEvent) {
        while self.events.len() >= self.events_capacity {
            if let Some(event) = self.events.pop_front() {
                log::warn!("Skipping event due to buffer overflow: {event:?}");
            } else {
                break;
            }
        }
        self.events.push_back(event);
        if let Some(waker) = self.waker.take() {
            waker.wake()
        }
    }
}

pub struct Input {
    state: Rc<RefCell<WindowState>>,
}

pub struct Window<'a> {
    pub raw: &'a RawWindow,
    pub input: Input,
}

impl<'a> Window<'a> {
    fn new(raw: &'a RawWindow, state: Rc<RefCell<WindowState>>) -> Self {
        Self {
            raw,
            input: Input { state },
        }
    }
}

pub fn create_window<T: 'static, F: AsyncFnOnce(Window<'_>) -> T + 'static>(
    app: AppProxy,
    attributes: WindowAttributes,
    event_loop: &ActiveEventLoop,
    window_main: F,
) -> Result<(TaskId, SharedCallState<T>), OsError> {
    let raw = event_loop.create_window(attributes)?;
    let id = raw.id();
    let state = Rc::new(RefCell::new(WindowState::new(None)));
    let weak = Rc::downgrade(&state);
    let (task, proxy) = app.create_task({
        let app = app.clone();
        async move {
            let window = Window::new(&raw, state.clone());
            let result = window_main(window).await;
            app.state.borrow_mut().remove_window(id);
            result
        }
    });
    app.state.borrow_mut().insert_window(id, task, weak);
    Ok((task, proxy))
}

impl Stream for Input {
    type Item = WindowEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut state = self.state.borrow_mut();
        match state.events.pop_front() {
            Some(event) => Poll::Ready(Some(event)),
            None => {
                if !state.terminated {
                    state.waker = Some(cx.waker().clone());
                    Poll::Pending
                } else {
                    Poll::Ready(None)
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.state.borrow().events.len(), None)
    }
}
