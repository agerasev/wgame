use alloc::rc::Rc;
use core::{
    cell::RefCell,
    mem,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use winit::{
    dpi::PhysicalSize,
    error::OsError,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window as WindowHandle, WindowAttributes},
};

use crate::{
    Runtime,
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
        if wake && let Some(waker) = self.waker.take() {
            waker.wake()
        }
    }
}

pub struct Window<'a> {
    handle: &'a WindowHandle,
    state: Rc<RefCell<WindowState>>,
    pub runtime: Runtime,
}

impl<'a> Window<'a> {
    fn new(handle: &'a WindowHandle, state: Rc<RefCell<WindowState>>, runtime: Runtime) -> Self {
        Self {
            handle,
            state,
            runtime,
        }
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

pub(crate) fn create_window<T: 'static, F: AsyncFnOnce(Window<'_>) -> T + 'static>(
    app: AppProxy,
    attributes: WindowAttributes,
    event_loop: &ActiveEventLoop,
    window_main: F,
    rt: Runtime,
) -> Result<(TaskId, SharedCallState<T>), OsError> {
    let handle = event_loop.create_window(update_attributes(attributes))?;
    let id = handle.id();
    let state = Rc::new(RefCell::new(WindowState::default()));
    let weak = Rc::downgrade(&state);
    let (task, proxy) = app.create_task({
        let app = app.clone();
        async move {
            let window = Window::new(&handle, state.clone(), rt);
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
        self.handle.request_redraw();
        WaitRedraw { owner: self }
    }
}

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
                    resized: state.resized.take(),
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
