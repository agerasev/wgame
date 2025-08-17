#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;
#[cfg(not(feature = "web"))]
extern crate std;

mod app;
mod executor;
mod proxy;
pub mod runtime;
pub mod timer;
pub mod window;

pub use crate::{app::App, runtime::Runtime, window::Window};

pub use winit::window::WindowAttributes;

pub mod deps {
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

#[macro_export]
macro_rules! run_app {
    ($crate_:path, $app_fn:expr) => {{
        use $crate_::{App, Runtime, deps::*};

        log::info!("Running App");
        let app = App::new().unwrap();
        let proxy = app.proxy();
        proxy.create_task($app_fn(Runtime::new(proxy.clone())));
        app.run().unwrap();
    }};
}

#[macro_export]
macro_rules! open_window {
    ($crate_:path, $window_fn:expr) => {{
        use $crate_::{App, Runtime, WindowAttributes, deps::*};

        async |rt: Runtime| {
            rt.create_windowed_task(WindowAttributes::default(), async move |window| {
                log::info!("Window created");
                $window_fn(window).await
            })
            .await
            .unwrap()
            .await
        }
    }};
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! entry {
    ($crate_:path, $main:ident, $app_fn:expr) => {
        pub fn $main() {
            use $crate_::{deps::*, run_app};

            env_logger::Builder::from_env(
                env_logger::Env::default().default_filter_or("info"), //
            )
            .init();

            run_app!($crate_, $app_fn);
        }
    };
}

#[cfg(feature = "web")]
#[macro_export]
macro_rules! entry {
    ($crate_:path, $main:ident, $app_fn:expr) => {
        pub fn $main() {
            use $crate_::{deps::*, run_app};

            console_error_panic_hook::set_once();
            console_log::init_with_level(log::Level::Info).unwrap();

            run_app!($crate_, $app_fn);
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
