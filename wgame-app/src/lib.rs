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
    runtime::{Runtime, sleep, spawn, within_window},
    window::Window,
};

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
macro_rules! run {
    ($crate_:path, $async_main:path) => {{
        use $crate_::{App, deps::*};

        log::info!("Running App");
        let app = App::new().unwrap();
        app.proxy().create_task::<_, ()>($async_main());
        app.run().unwrap();
    }};
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! entry {
    ($crate_:path, $main:ident, $async_main:path) => {
        pub fn $main() {
            use $crate_::{deps::*, run};

            env_logger::Builder::from_env(
                env_logger::Env::default().default_filter_or("info"), //
            )
            .init();

            run!($crate_, $async_main);
        }
    };
}

#[cfg(feature = "web")]
#[macro_export]
macro_rules! entry {
    ($crate_:path, $main:ident, $async_main:path) => {
        pub fn $main() {
            use $crate_::{deps::*, run};

            console_error_panic_hook::set_once();
            console_log::init_with_level(log::Level::Info).unwrap();

            run!($crate_, $async_main);
        }
    };
}

#[cfg(all(not(feature = "std"), not(feature = "web")))]
#[macro_export]
macro_rules! entry {
    ($crate_:path, $main:ident, $async_main:path) => {
        #![error("Neither `std` nor `web` feature enabled")]
    };
}

#[macro_export]
macro_rules! main {
    ($async_main:path) => {
        $crate::entry!($crate, main, $async_main);
    };
}
