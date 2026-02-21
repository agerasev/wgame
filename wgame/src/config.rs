//! Window configuration.

use crate::{
    app::{Size, WindowAttributes},
    gfx::{self, PresentMode},
};

/// Configuration for a window.
///
/// Combines application-level window attributes with graphics configuration.
#[derive(Clone, Default, Debug)]
pub struct WindowConfig {
    /// Window attributes from wgame_app.
    pub app: WindowAttributes,
    /// Graphics configuration.
    pub gfx: gfx::Config,
}

impl WindowConfig {
    /// Sets the window title.
    pub fn title(self, title: &str) -> Self {
        Self {
            app: self.app.with_title(title),
            ..self
        }
    }

    /// Sets the window size.
    pub fn size(self, size: (u32, u32)) -> Self {
        Self {
            app: self.app.with_inner_size(Size::new(size.0, size.1)),
            ..self
        }
    }

    /// Sets whether the window is resizable.
    pub fn resizable(self, resizable: bool) -> Self {
        Self {
            app: self.app.with_resizable(resizable),
            ..self
        }
    }

    /// Sets whether vsync is enabled.
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
