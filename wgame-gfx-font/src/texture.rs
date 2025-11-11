use std::ops::Deref;

use wgame_gfx_texture::{Texture, TextureAtlas};

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
    pub fn new(
        state: &TextState,
        font_atlas: &FontAtlas,
        texture_atlas: &TextureAtlas<u8>,
    ) -> Self {
        assert_eq!(&**state, &texture_atlas.state());
        if font_atlas.image().atlas() != texture_atlas.inner() {
            panic!("Font atlas and texture atlas are built upon different atlases");
        }
        let texture = Texture::new(texture_atlas, font_atlas.image());
        Self {
            library: state.clone(),
            atlas: font_atlas.clone(),
            texture,
        }
    }

    pub fn text(&self, text: &str) -> Text {
        Text::new(self, text)
    }

    pub fn inner(&self) -> &Texture<u8> {
        &self.texture
    }
}
