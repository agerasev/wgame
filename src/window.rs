use std::{
    cell::RefCell,
    mem,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use futures::{Stream, future::FusedFuture};
use winit::{
    error::OsError, event::WindowEvent, event_loop::ActiveEventLoop, window::WindowAttributes,
};

use crate::app::{AppProxy, WindowState};

pub struct Window {
    app: AppProxy,
    state: Rc<RefCell<WindowState>>,
}

impl Window {
    pub(crate) fn new(app: AppProxy, attributes: WindowAttributes) -> Self {
        let state = app.state.borrow_mut().create_window_state(attributes);
        Self { app, state }
    }

    pub(crate) fn create(&mut self) -> Create<'_> {
        Create {
            owner: Some(self),
            state: Rc::new(RefCell::new(CreateState { error: None })),
        }
    }

    pub fn render(&mut self) -> Render<'_> {
        Render {
            owner: Some(self),
            state: Rc::new(RefCell::new(CreateState { error: None })),
        }
    }

    pub fn events(&mut self) -> EventPipe<'_> {
        EventPipe { owner: self }
    }

    pub fn is_closed(&self) -> bool {
        self.state.borrow().close_requested
    }
    pub fn closed(&mut self) -> Closed<'_> {
        Closed { owner: self }
    }

    fn poll_create(&mut self, create: &Rc<RefCell<CreateState>>) -> Poll<Result<(), OsError>> {
        // Check if an error occured
        if let Some(err) = create.borrow_mut().error.take() {
            return Poll::Ready(Err(err));
        }

        // Check app is not suspended
        if !self.app.state.borrow().is_active() {
            return Poll::Pending;
        }

        // Try get dynamic window state
        if self.state.borrow().dynamic.is_none() {
            let app = self.app.state.clone();
            let state = self.state.clone();
            let create = create.clone();
            self.app
                .executor
                .borrow_mut()
                .add_loop_call(move |event_loop: &ActiveEventLoop| {
                    if let Err(err) = app
                        .borrow_mut()
                        .try_create_and_insert_window_if_not_exist(&state, event_loop)
                    {
                        create.borrow_mut().error = Some(err);
                    }
                    if let Some(waker) = state.borrow_mut().waker.take() {
                        waker.wake();
                    }
                });
            return Poll::Pending;
        }

        Poll::Ready(Ok(()))
    }

    fn poll_render(
        &mut self,
        create: &Rc<RefCell<CreateState>>,
    ) -> Poll<Result<Option<()>, OsError>> {
        // Try get dynamic window state
        let mut state = self.state.borrow_mut();
        let dynamic = if let Some(dynamic) = &mut state.dynamic {
            dynamic
        } else {
            drop(state);
            // Create window
            match self.poll_create(create) {
                Poll::Ready(Ok(())) => unreachable!(),
                other => return other.map_ok(|()| unreachable!()),
            }
        };

        // Whether redraw requested
        if !mem::replace(&mut dynamic.redraw_requested, false) {
            dynamic.window.request_redraw();
            return Poll::Pending;
        }

        // If window closed
        if state.close_requested {
            return Poll::Ready(Ok(None));
        }

        Poll::Ready(Ok(Some(())))
    }
}

struct CreateState {
    error: Option<OsError>,
}

pub struct Create<'a> {
    owner: Option<&'a mut Window>,
    state: Rc<RefCell<CreateState>>,
}

impl<'a> Future for Create<'a> {
    type Output = Result<(), OsError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let owner = self
            .owner
            .take()
            .expect("FusedFuture polled again after it returned Ready");

        let poll = owner.poll_create(&self.state);

        if poll.is_pending() {
            owner.state.borrow_mut().waker = Some(cx.waker().clone());
            self.owner = Some(owner);
        }

        poll
    }
}

impl FusedFuture for Create<'_> {
    fn is_terminated(&self) -> bool {
        self.owner.is_none()
    }
}

pub struct Render<'a> {
    owner: Option<&'a mut Window>,
    state: Rc<RefCell<CreateState>>,
}

impl<'a> Future for Render<'a> {
    type Output = Result<Option<()>, OsError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let owner = self
            .owner
            .take()
            .expect("FusedFuture polled again after it returned Ready");

        let poll = owner.poll_render(&self.state);

        if poll.is_pending() {
            owner.state.borrow_mut().waker = Some(cx.waker().clone());
            self.owner = Some(owner);
        }

        poll
    }
}

impl FusedFuture for Render<'_> {
    fn is_terminated(&self) -> bool {
        self.owner.is_none()
    }
}

pub struct EventPipe<'a> {
    owner: &'a mut Window,
}

impl Stream for EventPipe<'_> {
    type Item = WindowEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut state = self.owner.state.borrow_mut();
        match state.events.pop_front() {
            Some(event) => Poll::Ready(Some(event)),
            None => {
                if state.close_requested {
                    Poll::Ready(None)
                } else {
                    state.waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.owner.state.borrow().events.len();
        (len, Some(len))
    }
}

pub struct Closed<'a> {
    owner: &'a mut Window,
}

impl Future for Closed<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.owner.state.borrow_mut();
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
        self.owner.is_closed()
    }
}
