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

pub use log;

#[cfg(not(feature = "web"))]
pub use env_logger;

#[cfg(feature = "web")]
pub use console_error_panic_hook;
#[cfg(feature = "web")]
pub use console_log;
#[cfg(feature = "web")]
pub use wasm_bindgen;

#[macro_export]
macro_rules! run {
    ($crate_:path, $async_main:path) => {{
        use $crate_::{App, Runtime, log};
        log::info!("Running App");
        let app = App::new().unwrap();
        let proxy = app.proxy();
        proxy.create_task($async_main(Runtime::new(proxy.clone())));
        app.run().unwrap();
    }};
}

#[cfg(not(feature = "web"))]
#[macro_export]
macro_rules! entry {
    ($crate_:path, $main:ident, $async_main:path) => {
        pub fn $main() {
            use $crate_::{env_logger, log, run};

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
        pub mod __wgame_app_mod {
            use super::{/**/ $async_main};
            use $crate_::{console_error_panic_hook, console_log, log, run, wasm_bindgen};

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn $main() {
                console_error_panic_hook::set_once();
                console_log::init_with_level(log::Level::Info).unwrap();

                run!($crate_, $async_main);
            }
        }
    };
}

#[macro_export]
macro_rules! main {
    ($async_main:path) => {
        $crate::entry!($crate, main, $async_main);
    };
}
