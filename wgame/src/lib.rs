//! A modular framework for async 2D graphics applications.
//!
//! # Getting started
//!
//! Until a release is published, use a path dependency on the `wgame` crate in
//! this checkout:
//!
//! ```toml
//! [dependencies]
//! wgame = { path = "/path/to/checkout/wgame" }
//! ```
//!
//! ```no_run
//! use wgame::{Window, prelude::*, gfx::types::color};
//!
//! #[wgame::window(size = (800, 600), title = "Hello wgame")]
//! async fn main(mut window: Window<'_>) -> wgame::Result<()> {
//!     while let Some(mut frame) = window.next_frame().await? {
//!         frame.clear(color::BLACK);
//!         frame.present();
//!     }
//!     Ok(())
//! }
//! ```
//!
//! [`Window`] owns the graphics surface; each [`Frame`] borrows it. Create a
//! [`Library`] outside the frame loop to reuse graphics resources. See [`guide`]
//! for links to ownership, drawing, resource, and input contracts.
//!
//! # Features and platforms
//!
//! Defaults select `desktop`, `shapes`, `fs`, `image`, `typography`, and `utils`.
//! Optional content features are independently usable. `Library::load_texture`
//! requires `fs` + `image`; `Library::load_font` requires `fs` + `typography`.
//!
//! `desktop` enables native windowing and the Vulkan/GLES/Metal/DX12 backends as
//! appropriate for the target. `web` selects WebGL2: disable default features when
//! using it. Native and web runtime features cannot be combined. WebGPU is available
//! at the lower [`gfx`] layer but is not selected by the facade's `web` feature.

#![forbid(unsafe_code)]

mod config;
pub mod host;
mod host_input;
pub use host::{ContentFrame, WindowHost};
/// Input local to a drawing area, independent of the UI backend.
pub use wgame_input as canvas;
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

pub mod shader {
    //! Shader utilities and types.
    //!
    //! The facade's `Attribute` derive selects the `wgame::shader` path.
    //! For layout rules, see [`trait@wgame_shader::Attribute`].
    //!
    //! ```
    //! use wgame::shader::Attribute;
    //! #[derive(wgame::shader::Attribute)]
    //! struct InstanceData {
    //!     transform: wgame::glam::Mat4,
    //!     tint: wgame::glam::Vec4,
    //! }
    //! let layout = InstanceData::bindings().layout(0).unwrap();
    //! assert_eq!(layout.len(), 5); // four matrix columns plus tint
    //! assert_eq!(InstanceData::SIZE, 80);
    //! ```
    pub use wgame_shader::AttributeGlobal as Attribute;
    pub use wgame_shader::*;
}

/// File system operations.
#[cfg(feature = "fs")]
pub use wgame_fs as fs;

/// 2D shape rendering.
#[cfg(feature = "shapes")]
pub use wgame_gfx_shapes as shapes;

/// Solid primitives rendered through the shared shape infrastructure.
#[cfg(feature = "3d")]
pub use wgame_gfx_3d as shapes3d;

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
    pub use crate::{ContentFrame, WindowHost};
    pub use wgame_gfx::prelude::*;
    #[cfg(feature = "3d")]
    pub use wgame_gfx_3d::Shapes3d;
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

pub mod guide {
    //! Navigation to the contracts documented by each owning API.
    //!
    //! - Window ownership and presentation: [`crate::Window`], [`crate::Frame`].
    //! - Tasks and cancellation: [`mod@crate::app`].
    //! - Input streams: [`crate::Input`].
    //! - Drawing order and retained rendering: [`crate::gfx`].
    //! - Assets and shared helpers: [`crate::Library`], [`crate::texture`].
    //! - Vertex/instance byte layouts: [`crate::shader`].
}
