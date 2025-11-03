use std::ops::Deref;
use wgame_texture::{Texture, TextureAtlas};

use crate::{FontAtlas, Text, TextLibrary};

#[derive(Clone)]
pub struct FontTexture {
    pub(crate) library: TextLibrary,
    font: FontAtlas,
    texture: Texture<u8>,
}

impl Deref for FontTexture {
    type Target = FontAtlas;
    fn deref(&self) -> &Self::Target {
        &self.font
    }
}

impl FontTexture {
    pub fn new(
        library: &TextLibrary,
        font_atlas: FontAtlas,
        texture_atlas: &TextureAtlas<u8>,
    ) -> Self {
        if font_atlas.atlas.borrow().image().atlas() != texture_atlas.atlas() {
            panic!("Font atlas and texture atlas are built upon different atlases");
        }
        let texture =
            Texture::from_image(&library, font_atlas.image(), wgpu::TextureFormat::R8Uint);
        Self {
            library: library.clone(),
            font: font_atlas,
            texture,
        }
    }

    pub fn text(&self, text: &str) -> Text {
        Text::new(self, text)
    }

    pub fn texture(this: &FontTexture) -> &Texture<u8> {
        &this.texture
    }
}
