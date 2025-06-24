use std::{
    cell::RefCell,
    collections::VecDeque,
    mem,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use futures::{Stream, StreamExt, future::FusedFuture};
use winit::{
    event::WindowEvent,
    window::{Window as RawWindow, WindowId},
};

use crate::{app::AppState, proxy::AppProxy};

pub struct WindowState {
    pub waker: Option<Waker>,
    pub events: VecDeque<WindowEvent>,
    pub events_capacity: usize,
    pub redraw_requested: bool,
    pub close_requested: bool,
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
            redraw_requested: false,
            close_requested: false,
        }
    }

    pub fn push_event(&mut self, event: WindowEvent) {
        match &event {
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            WindowEvent::RedrawRequested => {
                self.redraw_requested = true;
            }
            _ => (),
        }

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

pub struct Window {
    raw: RawWindow,
    app: Rc<RefCell<AppState>>,
    state: Rc<RefCell<WindowState>>,
}

impl Drop for Window {
    fn drop(&mut self) {
        self.app.borrow_mut().remove_window(self.raw.id());
    }
}

impl Window {
    pub(crate) fn new(raw: RawWindow, app: AppProxy) -> Self {
        Self {
            raw,
            app: app.state.clone(),
            state: Rc::new(RefCell::new(WindowState::new(None))),
        }
    }

    pub fn id(&self) -> WindowId {
        self.raw.id()
    }

    pub fn raw(&self) -> &RawWindow {
        &self.raw
    }
    pub fn raw_mut(&mut self) -> &mut RawWindow {
        &mut self.raw
    }

    pub(crate) fn state(&self) -> &Rc<RefCell<WindowState>> {
        &self.state
    }

    pub fn events(&mut self) -> Events<'_> {
        Events {
            state: &mut self.state,
        }
    }

    pub fn request_redraw(&mut self) -> RedrawRequested<'_> {
        self.raw.request_redraw();
        RedrawRequested { state: &self.state }
    }

    pub fn is_closed(&self) -> bool {
        self.state.borrow().close_requested
    }
    pub fn closed(&mut self) -> Closed<'_> {
        Closed { state: &self.state }
    }
}

pub struct Events<'a> {
    state: &'a RefCell<WindowState>,
}

impl Iterator for Events<'_> {
    type Item = WindowEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.borrow_mut().events.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.state.borrow().events.len(), None)
    }
}

impl Stream for Events<'_> {
    type Item = WindowEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut state = self.state.borrow_mut();
        match state.events.pop_front() {
            Some(event) => Poll::Ready(Some(event)),
            None => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.state.borrow().events.len(), None)
    }
}

impl Stream for Window {
    type Item = WindowEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.events().poll_next_unpin(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.state.borrow().events.len(), None)
    }
}

pub struct RedrawRequested<'a> {
    state: &'a RefCell<WindowState>,
}

impl Future for RedrawRequested<'_> {
    type Output = Option<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.borrow_mut();
        if mem::replace(&mut state.redraw_requested, false) || state.close_requested {
            Poll::Ready(if state.close_requested {
                None
            } else {
                Some(())
            })
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct Closed<'a> {
    state: &'a RefCell<WindowState>,
}

impl Future for Closed<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.borrow_mut();
        if state.close_requested {
            Poll::Ready(())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl FusedFuture for Closed<'_> {
    fn is_terminated(&self) -> bool {
        self.state.borrow().close_requested
    }
}
