use std::{
    cell::RefCell,
    mem,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use futures::future::FusedFuture;
use winit::{
    dpi::PhysicalSize,
    error::OsError,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window as WindowHandle, WindowAttributes},
};

use crate::{
    executor::TaskId,
    proxy::{AppProxy, SharedCallState},
};

#[derive(Default)]
pub(crate) struct WindowState {
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
    handle: &'a WindowHandle,
    state: Rc<RefCell<WindowState>>,
}

impl<'a> Window<'a> {
    fn new(handle: &'a WindowHandle, state: Rc<RefCell<WindowState>>) -> Self {
        Self { handle, state }
    }
}

pub(crate) fn create_window<T: 'static, F: AsyncFnOnce(Window<'_>) -> T + 'static>(
    app: AppProxy,
    attributes: WindowAttributes,
    event_loop: &ActiveEventLoop,
    window_main: F,
) -> Result<(TaskId, SharedCallState<T>), OsError> {
    let handle = event_loop.create_window(attributes)?;
    let id = handle.id();
    let state = Rc::new(RefCell::new(WindowState::default()));
    let weak = Rc::downgrade(&state);
    let (task, proxy) = app.create_task({
        let app = app.clone();
        async move {
            let window = Window::new(&handle, state.clone());
            let result = window_main(window).await;
            app.state.borrow_mut().remove_window(id);
            result
        }
    });
    app.state.borrow_mut().insert_window(id, task, weak);
    Ok((task, proxy))
}

impl<'a> Window<'a> {
    pub fn size(&self) -> (u32, u32) {
        self.handle.inner_size().into()
    }

    pub fn handle(&self) -> &'a WindowHandle {
        self.handle
    }

    pub fn request_redraw(&mut self) -> WaitRedraw<'a, '_> {
        WaitRedraw { owner: Some(self) }
    }
}

pub struct WaitRedraw<'a, 'b> {
    owner: Option<&'b mut Window<'a>>,
}

impl<'a, 'b> Future for WaitRedraw<'a, 'b> {
    type Output = Option<Redraw<'b>>;

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
            Poll::Ready(Some(Redraw {
                handle: owner.handle,
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

impl<'a, 'b> FusedFuture for WaitRedraw<'a, 'b> {
    fn is_terminated(&self) -> bool {
        self.owner.is_none()
    }
}

pub struct Redraw<'b> {
    handle: &'b WindowHandle,
    resized: Option<PhysicalSize<u32>>,
}

impl Redraw<'_> {
    pub fn resized(&self) -> Option<(u32, u32)> {
        self.resized.as_ref().copied().map(|s| s.into())
    }

    pub fn pre_present(&mut self) {
        self.handle.pre_present_notify();
    }
}

impl<'b> Drop for Redraw<'b> {
    fn drop(&mut self) {
        self.handle.request_redraw();
    }
}
