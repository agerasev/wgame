use std::{
    cell::RefCell,
    mem,
    ops::{Deref, DerefMut},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use futures::future::FusedFuture;
use wgame_common::Surface;
use winit::{
    dpi::PhysicalSize,
    error::OsError,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window as InnerWindow, WindowAttributes},
};

use crate::{
    executor::TaskId,
    proxy::{AppProxy, SharedCallState},
};

#[derive(Default)]
pub struct WindowState {
    pub waker: Option<Waker>,
    pub resize: Option<PhysicalSize<u32>>,
    pub redraw_requested: bool,
    pub close_requested: bool,
    pub terminated: bool,
}

impl WindowState {
    pub fn push_event(&mut self, event: WindowEvent) {
        let mut wake = false;
        match event {
            WindowEvent::CloseRequested => {
                self.close_requested = true;
                wake = true;
            }
            WindowEvent::RedrawRequested => {
                self.redraw_requested = true;
                wake = true;
            }
            WindowEvent::Resized(size) => {
                self.resize = Some(size);
                wake = true;
            }
            _ => (),
        }
        if wake {
            if let Some(waker) = self.waker.take() {
                waker.wake()
            }
        }
    }
}

pub struct Window<'a> {
    inner: &'a InnerWindow,
    state: Rc<RefCell<WindowState>>,
}

impl<'a> Window<'a> {
    fn new(inner: &'a InnerWindow, state: Rc<RefCell<WindowState>>) -> Self {
        Self { inner, state }
    }

    pub fn inner(&self) -> &'a InnerWindow {
        self.inner
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
    let state = Rc::new(RefCell::new(WindowState::default()));
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

impl<'a> Window<'a> {
    pub fn next_frame<'b, S: Surface + 'b>(
        &'b mut self,
        surface: &'b mut S,
    ) -> WaitFrame<'a, 'b, S> {
        WaitFrame {
            owner: Some(self),
            surface: Some(surface),
        }
    }
}

pub struct WaitFrame<'a, 'b, S: Surface + 'b> {
    owner: Option<&'b mut Window<'a>>,
    surface: Option<&'b mut S>,
}

impl<'a, 'b, S: Surface + 'b> Future for WaitFrame<'a, 'b, S> {
    type Output = Result<Option<Frame<'a, 'b, S>>, S::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let owner = match self.owner.take() {
            Some(owner) => owner,
            None => panic!("Window terminated but its task polled"),
        };
        let surface = self.surface.take().unwrap();

        let mut state = owner.state.borrow_mut();
        if state.terminated {
            log::error!("Window terminated but its task polled");
            return Poll::Ready(Ok(None));
        }

        if state.close_requested {
            return Poll::Ready(Ok(None));
        }

        if let Some(size) = state.resize.take() {
            surface.resize(size.into())?;
        }

        if mem::take(&mut state.redraw_requested) {
            drop(state);
            let inner_frame = surface.create_frame()?;
            Poll::Ready(Ok(Some(Frame {
                owner,
                inner: Some(inner_frame),
            })))
        } else {
            state.waker = Some(cx.waker().clone());
            drop(state);
            self.owner = Some(owner);
            self.surface = Some(surface);
            Poll::Pending
        }
    }
}

impl<'a, 'b, S: Surface + 'b> FusedFuture for WaitFrame<'a, 'b, S> {
    fn is_terminated(&self) -> bool {
        self.owner.is_none()
    }
}

pub struct Frame<'a, 'b, S: Surface + 'b> {
    owner: &'b mut Window<'a>,
    inner: Option<S::Frame<'b>>,
}

impl<'a, 'b, S: Surface + 'b> Deref for Frame<'a, 'b, S> {
    type Target = S::Frame<'b>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl<'a, 'b, S: Surface + 'b> DerefMut for Frame<'a, 'b, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

impl<'a, 'b, S: Surface + 'b> Drop for Frame<'a, 'b, S> {
    fn drop(&mut self) {
        self.owner.inner.pre_present_notify();
        drop(self.inner.take());
        self.owner.inner.request_redraw();
    }
}
