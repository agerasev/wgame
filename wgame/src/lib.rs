//! High-level graphics application framework.
//!
//! This crate provides a high-level interface for creating graphics applications
//! using wgpu. It integrates various components for window management, rendering,
//! input handling, and resource loading.
//!
//! # Features
//!
//! - `desktop`: Enables desktop platform support (default)
//! - `web`: Enables web platform support
//! - `shapes`: Enables 2D shape rendering
//! - `fs`: Enables file system operations
//! - `image`: Enables image processing
//! - `typography`: Enables text rendering
//! - `utils`: Enables utility functions
//!
//! # Examples
//!
//! Basic application with a single window:
//!
//! ```no_run
//! use wgame::{Window, Result};
//!
//! #[wgame::window]
//! async fn main(window: Window<'_>) -> Result<()> {
//!     // Application logic here
//!     Ok(())
//! }
//! ```
//!
//! Application with custom window configuration:
//!
//! ```no_run
//! use wgame::{Window, WindowConfig, Result};
//!
//! #[wgame::window(width = 800, height = 600, title = "My App")]
//! async fn main(window: Window<'_>) -> Result<()> {
//!     // Application logic here
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]

mod config;
mod library;
mod window;

use std::{cell::RefCell, rc::Rc};

/// Re-export of wgame_app for convenience.
pub use wgame_app as app;

/// Re-export of wgame_gfx for convenience.
pub use wgame_gfx as gfx;

/// Re-export of wgame_gfx_texture for convenience.
pub use wgame_gfx_texture as texture;

/// Re-export of wgame_macros for convenience.
pub use wgame_macros::{app, window};

/// Shader utilities module.
///
/// This module provides re-exports of wgame_shader types with a shorter path.
pub mod shader {
    pub use wgame_shader::AttributeGlobal as Attribute;
    pub use wgame_shader::*;
}

/// File system operations (enabled with `fs` feature).
#[cfg(feature = "fs")]
pub use wgame_fs as fs;

/// 2D shape rendering (enabled with `shapes` feature).
#[cfg(feature = "shapes")]
pub use wgame_gfx_shapes as shapes;

/// Text rendering (enabled with `typography` feature).
#[cfg(feature = "typography")]
pub use wgame_gfx_typography as typography;

/// Image processing (enabled with `image` feature).
#[cfg(feature = "image")]
pub use wgame_image as image;

/// Utility functions (enabled with `utils` feature).
#[cfg(feature = "utils")]
pub use wgame_utils as utils;

/// Error type for wgame applications.
pub use anyhow::{Error, Result};

/// Re-exports from wgame_app for convenience.
pub use app::{Event, Input, Runtime, input, sleep, spawn};

/// Public items from config, library, and window modules.
pub use crate::{config::*, library::*, window::*};

/// Commonly used types and traits.
///
/// This module re-exports frequently used items from wgame_gfx and wgame_gfx_shapes
/// (when the shapes feature is enabled).
pub mod prelude {
    pub use wgame_gfx::prelude::*;
    #[cfg(feature = "shapes")]
    pub use wgame_gfx_shapes::prelude::*;
}

/// Re-export of glam for vector and matrix operations.
pub use glam;

/// Re-export of half for half-precision floating point types.
pub use half;

/// Re-export of rgb for color types.
pub use rgb;

/// Creates a main function that runs an application.
///
/// This macro generates a `main` function that calls `wgame::app::entry`
/// with the provided async function. It's used to set up the application
/// runtime and execute the main logic.
///
/// # Arguments
///
/// * `$main:ident` - The name of the main function to generate
/// * `$app_fn:expr` - The async function to run as the application
///
/// # Examples
///
/// ```no_run
/// use wgame::run_app;
///
/// async fn my_app() {
///     println!("Hello, world!");
/// }
///
/// run_app!(main, my_app);
/// ```
#[macro_export]
macro_rules! run_app {
    ($main:ident, $app_fn:expr) => {
        fn $main() {
            $crate::app::entry($app_fn);
        }
    };
}

/// Creates a main function that runs a windowed application.
///
/// This macro generates a `main` function that creates a single window
/// using the provided configuration and runs the window function.
/// It's a convenience wrapper around `run_app` and `app_with_single_window`.
///
/// # Arguments
///
/// * `$main:ident` - The name of the main function to generate
/// * `$window_fn:expr` - The async function to run in the window
/// * `$config:expr` - The window configuration
///
/// # Examples
///
/// ```no_run
/// use wgame::{run_window, Window, WindowConfig, Result};
///
/// async fn my_window(window: Window<'_>) -> Result<()> {
///     // Window logic here
///     Ok(())
/// }
///
/// run_window!(main, my_window, WindowConfig::default());
/// ```
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
///
/// This function creates a window with the specified configuration and runs
/// the provided window function. If the window is closed or suspended, it
/// will be recreated automatically.
///
/// # Arguments
///
/// * `window_fn` - The async function to run in the window
/// * `config` - The window configuration
///
/// # Returns
///
/// Returns the result of the window function when it completes successfully.
///
/// # Examples
///
/// ```no_run
/// use wgame::{app_with_single_window, Window, WindowConfig, Result};
///
/// async fn my_window(window: Window<'_>) -> Result<()> {
///     // Window logic here
///     Ok(())
/// }
///
/// #[wgame::app]
/// async fn main() {
///     app_with_single_window(my_window, WindowConfig::default()).await;
/// }
/// ```
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
