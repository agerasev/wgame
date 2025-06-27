#![forbid(unsafe_code)]

pub use wgame_app as app;
pub use wgame_common as common;

pub use app::{App, Runtime, run_main};

pub use wgame_macros::main;
