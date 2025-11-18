#![forbid(unsafe_code)]

mod config;
mod library;
mod window;

use std::{cell::RefCell, rc::Rc};

pub use wgame_app as app;
pub use wgame_gfx as gfx;
pub use wgame_gfx_texture as texture;
pub use wgame_macros::{app, window};
pub mod shader {
    pub use wgame_shader::AttributeGlobal as Attribute;
    pub use wgame_shader::*;
}

#[cfg(feature = "fs")]
pub use wgame_fs as fs;
#[cfg(feature = "font")]
pub use wgame_gfx_font as font;
#[cfg(feature = "shapes")]
pub use wgame_gfx_shapes as shapes;
#[cfg(feature = "image")]
pub use wgame_image as image;
#[cfg(feature = "utils")]
pub use wgame_utils as utils;

pub use anyhow::{Error, Result};
pub use app::{Event, Input, Runtime, input, sleep, spawn};

pub use crate::{config::*, library::*, window::*};

pub mod prelude {
    pub use wgame_gfx::prelude::*;
}

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
