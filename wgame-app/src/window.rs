use std::{
    cell::RefCell,
    mem,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use futures::future::FusedFuture;
use wgame_common::Window as CommonWindow;
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
    pub resized: Option<PhysicalSize<u32>>,
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
                self.resized = Some(size);
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
}

impl<'a> CommonWindow for Window<'a> {
    type Inner = &'a InnerWindow;

    fn inner(&self) -> Self::Inner {
        self.inner
    }

    fn size(&self) -> (u32, u32) {
        self.inner.inner_size().into()
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
    pub fn next_frame<'b>(&'b mut self) -> WaitFrame<'a, 'b> {
        WaitFrame { owner: Some(self) }
    }
}

pub struct WaitFrame<'a, 'b> {
    owner: Option<&'b mut Window<'a>>,
}

impl<'a, 'b> Future for WaitFrame<'a, 'b> {
    type Output = Option<Frame<'b>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let owner = match self.owner.take() {
            Some(owner) => owner,
            None => panic!("This WaitFrame has already returned Frame"),
        };

        let mut state = owner.state.borrow_mut();
        if state.terminated {
            log::error!("Window terminated but its task still alive");
            return Poll::Ready(None);
        }

        if state.close_requested {
            return Poll::Ready(None);
        }

        let resized = state.resized.take();

        if mem::take(&mut state.redraw_requested) {
            drop(state);
            Poll::Ready(Some(Frame {
                inner: owner.inner,
                resized,
            }))
        } else {
            state.waker = Some(cx.waker().clone());
            drop(state);
            self.owner = Some(owner);
            Poll::Pending
        }
    }
}

impl<'a, 'b> FusedFuture for WaitFrame<'a, 'b> {
    fn is_terminated(&self) -> bool {
        self.owner.is_none()
    }
}

pub struct Frame<'b> {
    inner: &'b InnerWindow,
    resized: Option<PhysicalSize<u32>>,
}

impl wgame_common::Frame for Frame<'_> {
    fn resized(&self) -> Option<(u32, u32)> {
        self.resized.as_ref().copied().map(|s| s.into())
    }

    fn pre_present(&mut self) {
        self.inner.pre_present_notify();
    }
}

impl<'b> Drop for Frame<'b> {
    fn drop(&mut self) {
        self.inner.request_redraw();
    }
}
