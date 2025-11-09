use std::ops::Deref;
use wgame_texture::{Texture, TextureAtlas};

use crate::{FontAtlas, Text, TextState};

#[derive(Clone)]
pub struct FontTexture {
    pub(crate) library: TextState,
    atlas: FontAtlas,
    texture: Texture<u8>,
}

impl Deref for FontTexture {
    type Target = FontAtlas;
    fn deref(&self) -> &Self::Target {
        &self.atlas
    }
}

impl FontTexture {
    pub fn new(state: &TextState, font: &FontAtlas, texture: &TextureAtlas<u8>) -> Self {
        assert_eq!(&**state, &texture.state());
        if font.atlas.borrow().image().atlas() != texture.atlas() {
            panic!("Font atlas and texture atlas are built upon different atlases");
        }
        let texture = Texture::from_image(&state, font.image(), wgpu::TextureFormat::R8Uint);
        Self {
            library: state.clone(),
            atlas: font.clone(),
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
