//! High-level graphics application framework built on WGPU and Winit.
//!
//! Provides window management, rendering, input handling, and resource loading for 2D graphics applications.

#![forbid(unsafe_code)]

mod config;
mod library;
mod window;

use std::{cell::RefCell, rc::Rc};

/// Application framework re-export.
pub use wgame_app as app;

/// Graphics rendering re-export.
pub use wgame_gfx as gfx;

/// Texture utilities re-export.
pub use wgame_gfx_texture as texture;

/// Macro re-exports for application and window creation.
pub use wgame_macros::{app, window};

/// Shader utilities and types.
pub mod shader {
    pub use wgame_shader::AttributeGlobal as Attribute;
    pub use wgame_shader::*;
}

/// File system operations.
#[cfg(feature = "fs")]
pub use wgame_fs as fs;

/// 2D shape rendering.
#[cfg(feature = "shapes")]
pub use wgame_gfx_shapes as shapes;

/// Text rendering.
#[cfg(feature = "typography")]
pub use wgame_gfx_typography as typography;

/// Image processing.
#[cfg(feature = "image")]
pub use wgame_image as image;

/// Utility functions.
#[cfg(feature = "utils")]
pub use wgame_utils as utils;

/// Error type for wgame applications.
pub use anyhow::{Error, Result};

/// Application framework items.
pub use app::{Event, Input, Runtime, input, sleep, spawn};

/// Public items from config, library, and window modules.
pub use crate::{config::*, library::*, window::*};

/// Commonly used types and traits.
pub mod prelude {
    pub use wgame_gfx::prelude::*;
    #[cfg(feature = "shapes")]
    pub use wgame_gfx_shapes::prelude::*;
}

/// Vector and matrix operations.
pub use glam;

/// Half-precision floating point types.
pub use half;

/// Color types.
pub use rgb;

/// Creates a main function that runs an application.
#[macro_export]
macro_rules! run_app {
    ($main:ident, $app_fn:expr) => {
        fn $main() {
            $crate::app::entry($app_fn);
        }
    };
}

/// Creates a main function that runs a windowed application.
#[macro_export]
macro_rules! run_window {
    ($main:ident, $window_fn:expr, $config:expr) => {
        $crate::run_app!($main, async || $crate::app_with_single_window(
            $window_fn, $config,
        )
        .await);
    };
}

/// Runs an application with a single window.
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
