//! Window configuration.
//!
//! This module provides types for configuring window properties and graphics settings.

use crate::{
    app::{Size, WindowAttributes},
    gfx::{self, PresentMode},
};

/// Configuration for a window.
///
/// This struct combines application-level window attributes with graphics
/// configuration. It provides builder-style methods for configuring various
/// window properties.
///
/// # Examples
///
/// ```no_run
/// use wgame::WindowConfig;
///
/// let config = WindowConfig::default()
///     .title("My Application")
///     .size(800, 600)
///     .resizable(true)
///     .vsync(true);
/// ```
#[derive(Clone, Default, Debug)]
pub struct WindowConfig {
    /// Window attributes from wgame_app.
    pub app: WindowAttributes,
    /// Graphics configuration.
    pub gfx: gfx::Config,
}

impl WindowConfig {
    /// Sets the window title.
    ///
    /// # Arguments
    ///
    /// * `title` - The title to set for the window
    ///
    /// # Returns
    ///
    /// Returns a new `WindowConfig` with the updated title.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wgame::WindowConfig;
    ///
    /// let config = WindowConfig::default().title("My App");
    /// ```
    pub fn title(self, title: &str) -> Self {
        Self {
            app: self.app.with_title(title),
            ..self
        }
    }

    /// Sets the window size.
    ///
    /// # Arguments
    ///
    /// * `size` - A tuple containing the width and height in pixels
    ///
    /// # Returns
    ///
    /// Returns a new `WindowConfig` with the updated size.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wgame::WindowConfig;
    ///
    /// let config = WindowConfig::default().size(800, 600);
    /// ```
    pub fn size(self, size: (u32, u32)) -> Self {
        Self {
            app: self.app.with_inner_size(Size::new(size.0, size.1)),
            ..self
        }
    }

    /// Sets whether the window is resizable.
    ///
    /// # Arguments
    ///
    /// * `resizable` - Whether the window should be resizable
    ///
    /// # Returns
    ///
    /// Returns a new `WindowConfig` with the updated resizable setting.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wgame::WindowConfig;
    ///
    /// let config = WindowConfig::default().resizable(true);
    /// ```
    pub fn resizable(self, resizable: bool) -> Self {
        Self {
            app: self.app.with_resizable(resizable),
            ..self
        }
    }

    /// Sets whether vsync is enabled.
    ///
    /// # Arguments
    ///
    /// * `vsync` - Whether vsync should be enabled
    ///
    /// # Returns
    ///
    /// Returns a new `WindowConfig` with the updated vsync setting.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wgame::WindowConfig;
    ///
    /// let config = WindowConfig::default().vsync(true);
    /// ```
    pub fn vsync(self, vsync: bool) -> Self {
        Self {
            gfx: gfx::Config {
                present_mode: if vsync {
                    PresentMode::AutoVsync
                } else {
                    PresentMode::AutoNoVsync
                },
            },
            ..self
        }
    }
}
