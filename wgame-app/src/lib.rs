#![forbid(unsafe_code)]

mod app;
mod executor;
mod proxy;
pub mod runtime;
mod timer;
pub mod window;

pub use crate::{app::App, runtime::Runtime, window::Window};
#[cfg(feature = "web")]
pub use wasm_bindgen;
pub use winit::window::WindowAttributes;

#[cfg(not(feature = "web"))]
#[macro_export]
macro_rules! run {
    ($main:ident, $async_main:path) => {
        pub fn $main() {
            let app = $crate::App::new().unwrap();
            let proxy = app.proxy();
            proxy.create_task($async_main(Runtime::new(proxy.clone())));
            app.run().unwrap();
        }
    };
}

#[cfg(feature = "web")]
#[macro_export]
macro_rules! run {
    ($main:ident, $async_main:path) => {
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn $main() {
            let app = $crate::App::new().unwrap();
            let proxy = app.proxy();
            proxy.create_task($async_main(Runtime::new(proxy.clone())));
            app.run().unwrap();
        }
    };
}
