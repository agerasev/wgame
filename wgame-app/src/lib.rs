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
    runtime::{Runtime, Task, sleep, spawn},
    window::Window,
    windowed_task::{WindowError, WindowedTask, create_windowed_task},
};
pub use winit::{dpi::PhysicalSize as Size, window::WindowAttributes};

use std::{cell::RefCell, fmt::Debug, rc::Rc};

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

#[cfg(all(not(feature = "std"), not(feature = "web")))]
pub fn entry<R, F>(app_fn: F)
where
    R: MainResult + 'static,
    F: AsyncFnOnce() -> R + 'static,
{
    #![error("Neither `std` nor `web` feature enabled")]
}

#[macro_export]
macro_rules! app_main {
    ($app_fn:expr) => {
        fn main() {
            $crate::entry($app_fn);
        }
    };
}

#[macro_export]
macro_rules! window_main {
    ($window_fn:expr) => {
        fn main() {
            $crate::entry(async || $crate::app_with_single_window($window_fn).await);
        }
    };
}
