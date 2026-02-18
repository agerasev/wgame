//! GPU-accelerated text rendering utilities.
//!
//! Provides font rasterization and text rendering with wgpu integration.

#![forbid(unsafe_code)]

mod library;
mod render;
mod text;
mod texture;

pub use self::{
    library::{TypographyLibrary, TypographyState},
    text::{Text, TextAlign},
    texture::FontTexture,
};

pub use wgame_typography::{Font as FontData, FontAtlas, RasterSettings, TextMetrics};

/// A font for GPU text rendering.
pub struct Font {
    lib: TypographyLibrary,
    data: FontData,
}

impl Font {
    /// Creates a new font for GPU rendering.
    pub fn new(lib: &TypographyLibrary, data: &FontData) -> Self {
        Self {
            lib: lib.clone(),
            data: data.clone(),
        }
    }

    /// Rasterizes the font with the given settings.
    pub fn rasterize(&self, settings: impl Into<RasterSettings>) -> FontTexture {
        self.lib.texture(&self.data, settings)
    }
}
