#![forbid(unsafe_code)]

mod library;
mod render;
mod text;
mod texture;

pub use self::{
    library::{TypographyLibrary, TypographyState},
    text::Text,
    texture::FontTexture,
};

pub use wgame_typography::{Font as FontData, FontAtlas};

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

    pub fn rasterize(&self, size: f32) -> FontTexture {
        self.lib.texture(&self.data, size)
    }
}
