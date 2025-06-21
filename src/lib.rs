#![forbid(unsafe_code)]

mod app;
mod executor;
pub mod runtime;
pub mod surface;
pub mod window;

pub use crate::{app::App, runtime::Runtime, window::Window};
pub use wgame_macros::main;
pub use winit::{
    error::{EventLoopError, OsError},
    window::WindowAttributes,
};

#[macro_export]
macro_rules! run_main {
    ($async_main:path) => {
        fn main() {
            let app = $crate::App::new().unwrap();
            let rt = app.runtime();
            rt.spawn($async_main(rt.clone()));
            app.run().unwrap();
        }
    };
}
