#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;
#[cfg(not(feature = "web"))]
extern crate std;

mod app;
mod executor;
mod output;
mod proxy;
pub mod runtime;
pub mod time;
pub mod window;

pub use crate::{
    app::App,
    runtime::{Runtime, WindowError, sleep, spawn, within_window},
    window::Window,
};

pub use winit::window::WindowAttributes;

use alloc::rc::Rc;
use core::{cell::RefCell, fmt::Debug};

use winit::error::OsError;

pub trait TryUnwrap {
    type Output;
    fn try_unwrap(self) -> Self::Output;
}

impl TryUnwrap for () {
    type Output = ();
    fn try_unwrap(self) {}
}

impl<T, E: Debug> TryUnwrap for Result<T, E> {
    type Output = T;
    fn try_unwrap(self) -> Self::Output {
        self.unwrap()
    }
}

pub fn run_app<R, F>(app_fn: F)
where
    R: TryUnwrap + 'static,
    F: AsyncFnOnce() -> R + 'static,
{
    log::info!("Running App");
    let app = App::new().unwrap();
    let output = app.proxy().create_task::<_, ()>(app_fn()).1;
    app.run().unwrap();
    output
        .try_take()
        .unwrap()
        .expect("Main task has been interrupted")
        .try_unwrap();
}

pub async fn app_with_single_window<R, F>(window_fn: F) -> Result<R, OsError>
where
    R: 'static,
    F: AsyncFnMut(Window) -> R + 'static,
{
    let window_fn = Rc::new(RefCell::new(window_fn));
    loop {
        let window_fn = window_fn.clone();
        let result = within_window(WindowAttributes::default(), async move |window| {
            log::info!("Window created");
            let result = (window_fn.borrow_mut())(window).await;
            log::info!("Window closed");
            result
        })
        .await;
        match result {
            Ok(x) => break Ok(x),
            Err(e) => match e {
                WindowError::Suspended => log::info!("Suspended"),
                WindowError::Other(e) => break Err(e),
            },
        }
    }
}

#[cfg(feature = "std")]
pub fn entry<R, F>(app_fn: F)
where
    R: TryUnwrap + 'static,
    F: AsyncFnOnce() -> R + 'static,
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    run_app(app_fn).try_unwrap()
}

#[cfg(feature = "web")]
pub fn entry<R, F>(app_fn: F)
where
    R: TryUnwrap + 'static,
    F: AsyncFnOnce() -> R + 'static,
{
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).unwrap();

    run_app(app_fn).try_unwrap();
}

#[cfg(all(not(feature = "std"), not(feature = "web")))]
pub fn entry<R, F>(app_fn: F)
where
    R: TryUnwrap + 'static,
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
