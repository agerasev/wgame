//! A cooperative event-loop runtime for windows, tasks, and timers.
//!
//! Work runs on the event-loop thread: blocking CPU work also blocks input and
//! rendering. Awaiting file I/O or timers yields control. A [`Runtime`] handle is
//! local to this thread.
//! Expired timers schedule another task poll even when there are no window or
//! input events; idle applications can use timers without continuous redraws.
//!
//! [`spawn`] creates a single-consumer [`Task`]. Use [`Task::handle`] for shared
//! cancellation and [`Task::cancel_on_drop`] for work owned by a window or scope.
//! [`WindowedTask`] similarly separates its result from its cancellation handle.
//! See those types for cancellation and suspension behavior.
//!
//! Native builds require an appropriate backend (`x11`/`wayland` on Linux, `std` on
//! Windows/macOS). Native and `web` runtime features cannot be enabled together.

#![forbid(unsafe_code)]

#[cfg(all(feature = "std", feature = "web"))]
compile_error!("`std` and `web` enabled at once");

mod app;
mod executor;
pub mod output;
pub mod runtime;
pub mod time;
pub mod window;
mod windowed_task;

pub use crate::{
    app::App,
    runtime::{Runtime, ScopedTask, Task, TaskHandle, sleep, spawn},
    window::Window,
    windowed_task::{WindowError, WindowTaskHandle, WindowedTask, create_windowed_task},
};
pub use wgame_app_input::{Event, Input};
pub use winit::{
    dpi::{LogicalSize, PhysicalSize as Size},
    window::{Window as RawWindow, WindowAttributes},
};
pub mod input {
    pub use wgame_app_input::{Event, Input, event, keyboard};
}

use std::{cell::RefCell, fmt::Debug, rc::Rc};

/// Trait for main function return types.
pub trait MainResult {
    fn try_unwrap(self);
}

impl MainResult for () {
    fn try_unwrap(self) {}
}

impl<E: Debug> MainResult for Result<(), E> {
    fn try_unwrap(self) {
        self.unwrap()
    }
}

/// Runs an application with the given async function.
pub fn run_app<R, F>(app_fn: F)
where
    R: MainResult + 'static,
    F: AsyncFnOnce() -> R + 'static,
{
    log::info!("Running App");
    let app = App::new().unwrap();
    let task = app.runtime().create_task(app_fn());
    app.run().unwrap();
    task.output()
        .try_take()
        .unwrap()
        .expect("Main task has been interrupted")
        .try_unwrap();
}

/// Runs an application with a single window.
#[allow(clippy::await_holding_refcell_ref)]
pub async fn app_with_single_window<R, F>(window_fn: F) -> R
where
    R: MainResult + 'static,
    F: AsyncFnMut(Window) -> R + 'static,
{
    let window_fn = Rc::new(RefCell::new(window_fn));
    loop {
        let window_fn = window_fn.clone();
        let result = create_windowed_task(
            &Runtime::current(),
            WindowAttributes::default(),
            async move |window| {
                log::info!("Window created");
                let result = (window_fn.borrow_mut())(window).await;
                log::info!("Window closed");
                result
            },
        )
        .await;
        match result {
            Ok(x) => break x,
            Err(e) => match e {
                WindowError::Creation(e) => panic!("Cannot create main window: {e}"),
                WindowError::Terminated => panic!("Main window terminated"),
                WindowError::Suspended => log::info!("Suspended"),
            },
        }
    }
}

/// Entry point for standard library builds.
#[cfg(feature = "std")]
#[allow(clippy::unit_arg)]
pub fn entry<R, F>(app_fn: F)
where
    R: MainResult + 'static,
    F: AsyncFnOnce() -> R + 'static,
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    run_app(app_fn).try_unwrap();
}

/// Entry point for web builds.
#[cfg(feature = "web")]
#[allow(clippy::unit_arg)]
pub fn entry<R, F>(app_fn: F)
where
    R: MainResult + 'static,
    F: AsyncFnOnce() -> R + 'static,
{
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).unwrap();

    run_app(app_fn).try_unwrap();
}

/// Entry point for builds without either `std` or `web` feature.
#[cfg(all(not(feature = "std"), not(feature = "web")))]
pub fn entry<R, F>(_app_fn: F)
where
    R: MainResult + 'static,
    F: AsyncFnOnce() -> R + 'static,
{
    compile_error!("Enable a native backend (x11/wayland/android/std) or web");
}

/// Macro to generate a main function for an application.
#[macro_export]
macro_rules! app_main {
    ($app_fn:expr) => {
        fn main() {
            $crate::entry($app_fn);
        }
    };
}

/// Macro to generate a main function for a windowed application.
#[macro_export]
macro_rules! window_main {
    ($window_fn:expr) => {
        fn main() {
            $crate::entry(async || $crate::app_with_single_window($window_fn).await);
        }
    };
}
