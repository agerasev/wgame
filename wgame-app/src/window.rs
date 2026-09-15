use std::{
    cell::RefCell,
    mem::replace,
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
    scale_factor: f64,
    scale_redraw: bool,
    close_requested: bool,
    redraw_requested: bool,
    redraw_ready: bool,
    terminated: bool,
    focused: bool,
    mouse_motion: (f64, f64),
}

impl WindowState {
    pub(crate) fn mouse_motion(&mut self, delta: (f64, f64)) {
        if self.focused && !self.terminated && delta.0.is_finite() && delta.1.is_finite() {
            self.mouse_motion.0 += delta.0;
            self.mouse_motion.1 += delta.1;
        }
    }

    pub fn push_event(&mut self, event: WindowEvent) {
        let mut wake = true;
        match &event {
            WindowEvent::Focused(focused) => {
                self.focused = *focused;
                self.mouse_motion = (0.0, 0.0);
                wake = false;
            }
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            WindowEvent::RedrawRequested => {
                self.redraw_ready = true;
            }
            WindowEvent::Resized(size) => {
                self.mouse_motion = (0.0, 0.0);
                self.size = *size;
                self.resized = true;
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor_changed(*scale_factor);
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

    fn scale_factor_changed(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
        self.scale_redraw = true;
    }

    pub fn terminate(&mut self) {
        self.terminated = true;
        self.handler.terminate();
        if let Some(waker) = self.waker.take() {
            waker.wake();
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
    let state = Rc::new(RefCell::new(WindowState {
        size: handle.inner_size(),
        scale_factor: handle.scale_factor(),
        focused: handle.has_focus(),
        resized: true,
        ..Default::default()
    }));
    let weak = Rc::downgrade(&state);
    struct Registration {
        app: Runtime,
        id: winit::window::WindowId,
    }
    impl Drop for Registration {
        fn drop(&mut self) {
            self.app.state.borrow_mut().remove_window(self.id);
        }
    }
    // Construct outside the future so cancellation before its first poll also cleans up.
    let registration = Registration {
        app: app.clone(),
        id,
    };
    let task = app.create_task({
        async move {
            let _registration = registration;
            let window = Window::new(&handle, state.clone());
            window_main(window).await
        }
    });
    app.state.borrow_mut().insert_window(id, task.id(), weak);
    Ok(task)
}

impl<'a> Window<'a> {
    /// Inner dimensions in physical pixels.
    pub fn size(&self) -> (u32, u32) {
        self.handle.inner_size().into()
    }

    /// OS scaling from logical pixels to physical pixels for this window.
    ///
    /// This may change when moving between monitors or changing desktop scaling.
    pub fn scale_factor(&self) -> f64 {
        self.handle.scale_factor()
    }

    pub fn raw(&self) -> &'a WindowHandle {
        self.handle
    }

    pub fn input(&self) -> Input {
        self.state.borrow_mut().handler.input()
    }

    pub fn request_redraw(&mut self) -> WaitRedraw<'a, '_> {
        self.handle.request_redraw();
        self.state.borrow_mut().redraw_requested = true;
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
    type Output = Option<Redraw<'a>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let owner = &mut self.owner;
        // A scale change can require fresh content without a physical resize.
        let scale_redraw = replace(&mut owner.state.borrow_mut().scale_redraw, false);
        if scale_redraw {
            owner.handle.request_redraw();
        }
        let mut state = owner.state.borrow_mut();
        if state.terminated {
            log::error!("Window terminated but its task still alive");
            return Poll::Ready(None);
        }

        if state.close_requested {
            state.close_requested = false;
            return Poll::Ready(None);
        }

        let result = if state.redraw_requested && state.redraw_ready {
            state.redraw_ready = false;
            if let size @ ((0, _) | (_, 0)) = owner.size() {
                log::warn!("Redraw requested but window size is zero: {size:?}");
                owner.handle.request_redraw();
                Poll::Pending
            } else {
                state.redraw_requested = false;
                Poll::Ready(Some(Redraw {
                    handle: owner.handle,
                    size: state.size,
                    scale_factor: state.scale_factor,
                    resized: replace(&mut state.resized, false),
                    mouse_motion: replace(&mut state.mouse_motion, (0.0, 0.0)),
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
    mouse_motion: (f64, f64),
    handle: &'b WindowHandle,
    size: PhysicalSize<u32>,
    scale_factor: f64,
    resized: bool,
}

impl Redraw<'_> {
    /// Accumulated raw device motion since the preceding redraw, while focused.
    /// Units are platform-defined device units, unaffected by display scaling.
    /// Useful with cursor locking; focus/size changes discard accumulated motion.
    pub fn mouse_motion(&self) -> (f64, f64) {
        self.mouse_motion
    }

    /// OS scale factor captured with this redraw's physical dimensions.
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    /// Inner dimensions in logical pixels, retaining fractional sizes.
    pub fn logical_size(&self) -> (f64, f64) {
        self.size.to_logical::<f64>(self.scale_factor).into()
    }

    /// Inner dimensions in physical pixels.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_motion_accumulates_only_while_focused_and_resets_on_focus_loss() {
        let mut state = WindowState::default();
        state.mouse_motion((3.0, 4.0));
        assert_eq!(state.mouse_motion, (0.0, 0.0));
        state.push_event(WindowEvent::Focused(true));
        state.mouse_motion((3.0, 4.0));
        state.mouse_motion((-1.0, 2.0));
        assert_eq!(replace(&mut state.mouse_motion, (0.0, 0.0)), (2.0, 6.0));
        state.mouse_motion((f64::NAN, 2.0));
        assert_eq!(state.mouse_motion, (0.0, 0.0));
        state.mouse_motion((3.0, 4.0));
        state.push_event(WindowEvent::Focused(false));
        assert_eq!(state.mouse_motion, (0.0, 0.0));
    }

    #[test]
    fn scale_changes_request_redraw_without_claiming_a_resize() {
        let mut state = WindowState {
            size: PhysicalSize::new(1800, 1200),
            scale_factor: 1.0,
            ..Default::default()
        };
        for factor in [1.25, 2.0, 1.0] {
            state.scale_factor_changed(factor);
            assert!(replace(&mut state.scale_redraw, false));
            assert_eq!(state.scale_factor, factor);
            assert_eq!(state.size, PhysicalSize::new(1800, 1200));
            assert!(!state.resized);
        }
        state.push_event(WindowEvent::Resized(PhysicalSize::new(900, 600)));
        assert!(state.resized);
        assert_eq!(state.scale_factor, 1.0);
    }
}
