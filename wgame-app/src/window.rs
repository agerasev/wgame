use std::{
    cell::RefCell,
    mem::{self, replace},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use wgame_app_input::{EventHandler, Input};
use winit::{
    dpi::PhysicalSize,
    error::OsError,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window as WindowHandle, WindowAttributes},
};

use crate::runtime::{Runtime, Task};

#[derive(Default)]
pub(crate) struct WindowState {
    handler: EventHandler,
    waker: Option<Waker>,
    size: PhysicalSize<u32>,
    resized: bool,
    close_requested: bool,
    redraw_requested: bool,
    terminated: bool,
}

impl WindowState {
    pub fn push_event(&mut self, event: WindowEvent) {
        let mut wake = true;
        match &event {
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            WindowEvent::RedrawRequested => {
                self.redraw_requested = true;
            }
            WindowEvent::Resized(size) => {
                self.size = *size;
                self.resized = true;
            }
            _ => {
                wake = false;
            }
        }
        if wake && let Some(waker) = self.waker.take() {
            waker.wake()
        }

        self.handler.push(event);
    }

    pub fn terminate(&mut self) {
        self.terminated = true;
        self.handler.terminate();
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

fn update_attributes(attributes: WindowAttributes) -> WindowAttributes {
    #[cfg(not(feature = "web"))]
    {
        attributes
    }
    #[cfg(feature = "web")]
    {
        use web_sys::wasm_bindgen::JsCast;
        use winit::platform::web::WindowAttributesExtWebSys;

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let html_canvas_element = canvas.unchecked_into();
        attributes.with_canvas(Some(html_canvas_element))
    }
}

pub(crate) fn create_window<T, F>(
    app: Runtime,
    attributes: WindowAttributes,
    event_loop: &ActiveEventLoop,
    window_main: F,
) -> Result<Task<T>, OsError>
where
    T: 'static,
    F: AsyncFnOnce(Window<'_>) -> T + 'static,
{
    let handle = event_loop.create_window(update_attributes(attributes))?;
    let id = handle.id();
    let state = Rc::new(RefCell::new(WindowState::default()));
    let weak = Rc::downgrade(&state);
    let task = app.create_task({
        let app = app.clone();
        async move {
            let window = Window::new(&handle, state.clone());
            let result = window_main(window).await;
            app.state.borrow_mut().remove_window(id);
            result
        }
    });
    app.state.borrow_mut().insert_window(id, task.id(), weak);
    Ok(task)
}

impl<'a> Window<'a> {
    pub fn size(&self) -> (u32, u32) {
        self.handle.inner_size().into()
    }

    pub fn raw(&self) -> &'a WindowHandle {
        self.handle
    }

    pub fn input(&self) -> Input {
        self.state.borrow_mut().handler.input()
    }

    pub fn request_redraw(&mut self) -> WaitRedraw<'a, '_> {
        self.handle.request_redraw();
        WaitRedraw { owner: self }
    }
}

/// Future to wait for window to be ready for redrawing.
///
/// Returns redraw handle [`Redraw`] or `None` if window requested to be closed.
///
/// If you want to ignore close request you can continue to poll this future.
/// Or you can safely drop it and [`request_redraw`](`Window::request_redraw`) again.
pub struct WaitRedraw<'a, 'b> {
    owner: &'b mut Window<'a>,
}

impl<'a, 'b> Future for WaitRedraw<'a, 'b> {
    type Output = Option<Redraw<'b>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let owner = &mut self.owner;
        let mut state = owner.state.borrow_mut();
        if state.terminated {
            log::error!("Window terminated but its task still alive");
            return Poll::Ready(None);
        }

        if state.close_requested {
            state.close_requested = false;
            return Poll::Ready(None);
        }

        let result = if mem::take(&mut state.redraw_requested) {
            if let size @ ((0, _) | (_, 0)) = owner.size() {
                log::warn!("Redraw requested but window size is zero: {size:?}");
                owner.handle.request_redraw();
                Poll::Pending
            } else {
                Poll::Ready(Some(Redraw {
                    handle: owner.handle,
                    size: state.size,
                    resized: replace(&mut state.resized, false),
                }))
            }
        } else {
            Poll::Pending
        };

        if result.is_pending() {
            state.waker = Some(cx.waker().clone());
        }

        result
    }
}

pub struct Redraw<'b> {
    handle: &'b WindowHandle,
    size: PhysicalSize<u32>,
    resized: bool,
}

impl Redraw<'_> {
    pub fn size(&self) -> (u32, u32) {
        self.size.into()
    }

    pub fn resized(&self) -> Option<(u32, u32)> {
        if self.resized {
            Some(self.size())
        } else {
            None
        }
    }

    pub fn pre_present(&mut self) {
        self.handle.pre_present_notify();
    }
}
