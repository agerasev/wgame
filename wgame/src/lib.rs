#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

mod config;
mod window;

use core::cell::RefCell;

use alloc::rc::Rc;
pub use wgame_app as app;
pub use wgame_gfx as gfx;
pub use wgame_macros::{app, window};

#[cfg(feature = "fs")]
pub use wgame_fs as fs;
#[cfg(feature = "img")]
pub use wgame_img as img;
#[cfg(feature = "shapes")]
pub use wgame_shapes as shapes;
#[cfg(feature = "text")]
pub use wgame_text as text;
#[cfg(feature = "utils")]
pub use wgame_utils as utils;

pub use anyhow::{Error, Result};
pub use app::{Runtime, sleep, spawn};

pub use crate::{config::*, window::*};

#[macro_export]
macro_rules! run_app {
    ($main:ident, $app_fn:expr) => {
        fn $main() {
            $crate::app::entry($app_fn);
        }
    };
}

#[macro_export]
macro_rules! run_window {
    ($main:ident, $window_fn:expr, $config:expr) => {
        $crate::run_app!($main, async || $crate::app_with_single_window(
            $window_fn, $config,
        )
        .await);
    };
}

#[allow(clippy::await_holding_refcell_ref)]
pub async fn app_with_single_window<R, F>(window_fn: F, config: WindowConfig) -> R
where
    R: app::MainResult + 'static,
    F: AsyncFnMut(Window) -> R + 'static,
{
    let window_fn = Rc::new(RefCell::new(window_fn));
    loop {
        let window_fn = window_fn.clone();
        let result =
            create_windowed_task(&Runtime::current(), config.clone(), async move |window| {
                log::info!("Window created");
                let result = (window_fn.borrow_mut())(window).await;
                log::info!("Window closed");
                result
            })
            .await;
        match result {
            Ok(r) => match r {
                Ok(x) => break x,
                Err(e) => panic!("Cannot initialize graphics: {e}"),
            },
            Err(e) => match e {
                WindowError::Creation(e) => panic!("Cannot create main window: {e}"),
                WindowError::Terminated => panic!("Main window terminated"),
                WindowError::Suspended => log::info!("Suspended"),
            },
        }
    }
}
