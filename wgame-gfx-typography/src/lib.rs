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

pub struct Font {
    lib: TypographyLibrary,
    data: FontData,
}

impl Font {
    pub fn new(lib: &TypographyLibrary, data: &FontData) -> Self {
        Self {
            lib: lib.clone(),
            data: data.clone(),
        }
    }

    pub fn rasterize(&self, settings: impl Into<RasterSettings>) -> FontTexture {
        self.lib.texture(&self.data, settings)
    }
}
