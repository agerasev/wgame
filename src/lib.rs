#![forbid(unsafe_code)]

pub use wgame_app as app;
pub use wgame_common as common;

pub use app::{App, Runtime, run};
pub use common::Frame;

#[cfg(feature = "web")]
pub use app::wasm_bindgen;
pub use wgame_macros::main;
