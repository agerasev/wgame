//! GPU text rendering with shaping, alignment, and cached monochrome glyphs.
//!
//! Load a [`Font`] once, rasterize it for a pixel size, and reuse the [`FontTexture`]
//! for labels. Cache unchanged [`Text`] objects outside the frame loop; recreate
//! size-dependent rasters after resizing when appropriate.
//!
//! Text respects glyph placement offsets from [`wgame_typography`]; see that crate
//! for rasterization and layout limitations.

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
