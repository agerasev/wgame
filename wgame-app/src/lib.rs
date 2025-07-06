#![forbid(unsafe_code)]

mod app;
mod executor;
mod proxy;
pub mod runtime;
pub mod timer;
pub mod window;

pub use crate::{app::App, runtime::Runtime, window::Window};
#[cfg(feature = "web")]
pub use wasm_bindgen;
pub use winit::window::WindowAttributes;

#[macro_export]
macro_rules! run {
    ($crate_:path, $async_main:path) => {{
        console_error_panic_hook::set_once();
        use $crate_::{App, Runtime};
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
            use $crate_::{/**/ run};
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
            use $crate_::{run, wasm_bindgen};

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn $main() {
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
