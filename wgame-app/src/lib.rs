#![forbid(unsafe_code)]

mod app;
mod executor;
mod proxy;
pub mod runtime;
mod timer;
pub mod window;

pub use crate::{app::App, runtime::Runtime, window::Window};
pub use winit::window::WindowAttributes;

#[macro_export]
macro_rules! run_main {
    ($async_main:path) => {
        fn main() {
            let app = $crate::App::new().unwrap();
            let proxy = app.proxy();
            proxy.create_task($async_main(Runtime::new(proxy.clone())));
            app.run().unwrap();
        }
    };
}
