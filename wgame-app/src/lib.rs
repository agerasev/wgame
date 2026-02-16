//! Application framework for wgame.
//!
//! This crate provides an async-based application framework built on top of `winit`
//! with support for windowing, events, timers, and task spawning. It is designed
//! to work in both standard (`std`) and `no_std` (web) environments.
//!
//! # Core Concepts
//!
//! ## Runtime and Task System
//!
//! The application framework uses an async runtime that integrates with the
//! windowing event loop. Tasks are spawned using [`spawn`] or via the
//! [`Runtime::create_task`] method.
//!
//! ```no_run
//! # use wgame_app::spawn;
//! # async fn example_task() {
//! # }
//! # async fn example() {
//! spawn(async {
//!     example_task().await;
//! });
//! # }
//! ```
//!
//! ## Windows
//!
//! Windows are created using [`create_windowed_task`] which returns a
//! [`WindowedTask`] that can be awaited for window lifecycle events.
//!
//! ```no_run
//! # use wgame_app::{create_windowed_task, WindowAttributes};
//! # async fn window_main(window: wgame_app::Window) -> () {
//! # }
//! # async fn example() {
//! let task = create_windowed_task(
//!     &wgame_app::Runtime::current(),
//!     WindowAttributes::default(),
//!     window_main
//! );
//! # }
//! ```
//!
//! ## Timers
//!
//! Timers can be created using [`sleep`] and [`sleep_until`] for delayed execution.
//!
//! ```no_run
//! # use wgame_app::sleep;
//! # use std::time::Duration;
//! # async fn example() {
//! sleep(Duration::from_secs(1)).await;
//! # }
//! ```
//!
//! # Entry Points
//!
//! The crate provides two main entry point macros:
//! - [`app_main!`] - For simple applications with a single async function
//! - [`window_main!`] - For applications that need to handle windows
//!
//! # Platform Support
//!
//! The crate supports both desktop (via `std`) and web (via `web` feature) targets.
//! Only one of these features can be enabled at a time.
//!
//! # Modules
//!
//! - [`app`] - Main application state and event loop handler
//! - [`executor`] - Async task executor
//! - [`output`] - Task output handling
//! - [`runtime`] - Runtime handle and task management
//! - [`time`] - Timer and time management
//! - [`window`] - Window management and event handling
//! - [`windowed_task`] - Task wrapper for window lifecycle

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
pub use wgame_app_input::{Event, Input};
pub use winit::{dpi::PhysicalSize as Size, window::WindowAttributes};
pub mod input {
    pub use wgame_app_input::{Event, Input, event, keyboard};
}

use std::{cell::RefCell, fmt::Debug, rc::Rc};

/// Trait for main function return types.
///
/// This trait allows the main function to return either `()` or a `Result<T, E>`.
/// The `try_unwrap` method will unwrap the result if it's an error, panicking
/// with a debug representation of the error.
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
///
/// This function creates a new application, runs the provided async function
/// as a task, and then starts the event loop. The function will block until
/// the task completes or the application exits.
///
/// # Examples
///
/// ```no_run
/// # use wgame_app::run_app;
/// async fn main_task() {
///     println!("Hello, world!");
/// }
///
/// run_app(main_task);
/// ```
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
///
/// This function creates a window using the provided attributes, runs the
/// window function, and handles window lifecycle events. If the window is
/// closed, it will be recreated.
///
/// # Examples
///
/// ```no_run
/// # use wgame_app::app_with_single_window;
/// # use winit::window::WindowAttributes;
/// async fn window_task(window: wgame_app::Window) -> () {
///     println!("Window created with size: {:?}", window.size());
/// }
///
/// app_with_single_window(window_task).await;
/// ```
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
///
/// This function initializes logging and runs the application. It should be
/// used as the main function in desktop applications.
///
/// # Examples
///
/// ```no_run
/// # use wgame_app::entry;
/// async fn main_task() {
///     println!("Hello, world!");
/// }
///
/// entry(main_task);
/// ```
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
///
/// This function initializes logging for web assembly and runs the application.
/// It should be used as the main function in web applications.
///
/// # Examples
///
/// ```no_run
/// # use wgame_app::entry;
/// # async fn main_task() {
/// #     println!("Hello, world!");
/// # }
/// # #[wasm_bindgen(start)]
/// # pub fn main() {
/// entry(main_task);
/// # }
/// ```
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
///
/// This function will cause a compile error if neither `std` nor `web` feature
/// is enabled.
#[cfg(all(not(feature = "std"), not(feature = "web")))]
pub fn entry<R, F>(_app_fn: F)
where
    R: MainResult + 'static,
    F: AsyncFnOnce() -> R + 'static,
{
    #![error("Neither `std` nor `web` feature enabled")]
}

/// Macro to generate a main function for an application.
///
/// This macro takes an async function expression and generates a `main` function
/// that calls [`entry`] with that function.
///
/// # Examples
///
/// ```
/// # use wgame_app::app_main;
/// app_main!(async {
///     println!("Hello, world!");
/// });
/// ```
#[macro_export]
macro_rules! app_main {
    ($app_fn:expr) => {
        fn main() {
            $crate::entry($app_fn);
        }
    };
}

/// Macro to generate a main function for a windowed application.
///
/// This macro takes an async function expression that accepts a [`Window`] and
/// generates a `main` function that creates a single window and runs the function.
///
/// # Examples
///
/// ```
/// # use wgame_app::window_main;
/// # async fn window_task(window: wgame_app::Window) -> () { }
/// window_main!(window_task);
/// ```
#[macro_export]
macro_rules! window_main {
    ($window_fn:expr) => {
        fn main() {
            $crate::entry(async || $crate::app_with_single_window($window_fn).await);
        }
    };
}
