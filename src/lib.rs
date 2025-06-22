#![forbid(unsafe_code)]

pub use wgame_app::*;
pub use wgame_macros::main;

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
