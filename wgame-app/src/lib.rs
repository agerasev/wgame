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

use core::fmt::Debug;

pub use crate::{
    app::App,
    runtime::{Runtime, sleep, spawn, within_window},
    window::Window,
};

pub use winit::window::WindowAttributes;

pub mod deps {
    pub use futures;
    pub use log;

    #[cfg(feature = "std")]
    pub use env_logger;

    #[cfg(feature = "web")]
    pub use console_error_panic_hook;
    #[cfg(feature = "web")]
    pub use console_log;
    #[cfg(feature = "web")]
    pub use wasm_bindgen;
}

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

#[macro_export]
macro_rules! run_app {
    ($crate_:path, $app_fn:expr) => {{
        use $crate_::{App, Runtime, deps::*};

        log::info!("Running App");
        let app = App::new().unwrap();
        let proxy = app.proxy();
        let output = app.proxy().create_task::<_, ()>($app_fn()).1;
        app.run().unwrap();
        output
            .try_take()
            .unwrap()
            .expect("Main task has been interrupted")
    }};
}

#[macro_export]
macro_rules! open_window {
    ($crate_:path, $window_fn:expr) => {{
        use $crate_::{App, WindowAttributes, deps::*, within_window};

        async || {
            within_window(WindowAttributes::default(), async move |window| {
                log::info!("Window created");
                let result = $window_fn(window).await;
                log::info!("Window closed");
                result
            })
            .await
        }
    }};
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! entry {
    ($crate_:path, $main:ident, $app_fn:expr) => {
        pub fn $main() {
            use $crate_::{TryUnwrap, deps::*, run_app};

            env_logger::Builder::from_env(
                env_logger::Env::default().default_filter_or("info"), //
            )
            .init();

            run_app!($crate_, $app_fn).try_unwrap();
        }
    };
}

#[cfg(feature = "web")]
#[macro_export]
macro_rules! entry {
    ($crate_:path, $main:ident, $app_fn:expr) => {
        pub fn $main() {
            use $crate_::{TryUnwrap, deps::*, run_app};

            console_error_panic_hook::set_once();
            console_log::init_with_level(log::Level::Info).unwrap();

            run_app!($crate_, $app_fn).try_unwrap();
        }
    };
}

#[cfg(all(not(feature = "std"), not(feature = "web")))]
#[macro_export]
macro_rules! entry {
    ($crate_:path, $main:ident, $app_fn:expr) => {
        #![error("Neither `std` nor `web` feature enabled")]
    };
}

#[macro_export]
macro_rules! app_main {
    ($app_fn:path) => {
        $crate::entry!($crate, main, $app_fn);
    };
}

#[macro_export]
macro_rules! window_main {
    ($window_fn:path) => {
        $crate::entry!($crate, main, $crate::open_window!($crate, $window_fn));
    };
}
