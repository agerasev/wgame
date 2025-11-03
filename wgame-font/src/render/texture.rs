use std::{
    cell::RefCell,
    cmp::Ordering,
    hash::{Hash, Hasher},
    ops::Deref,
    rc::Rc,
};

use etagere::euclid::default::Box2D;
use wgame_texture::Texture;
use wgpu::Extent3d;

use wgame_gfx::Graphics;

use crate::{Style, Text, TextLibrary, style::StyleAtlas};

#[derive(Clone)]
pub struct FontTexture {
    pub(crate) library: TextLibrary,
    style: Style,
    texture: Texture<u8>,
}

impl Deref for FontTexture {
    type Target = Style;
    fn deref(&self) -> &Self::Target {
        &self.style
    }
}

impl FontTexture {
    pub fn new(library: &TextLibrary, raster: Style) -> Self {
        Self {
            style: raster,
            library: library.clone(),
            texture: Rc::new(RefCell::new(None)),
        }
    }

    pub fn sync(&self) -> Option<wgpu::TextureView> {
        let mut texture = self.texture.borrow_mut();
        Texture::sync(
            &mut texture,
            &self.library,
            &mut self.style.atlas.borrow_mut(),
        );
        texture.as_ref().map(|t| t.view.clone())
    }

    pub fn text(&self, text: &str) -> Text {
        Text::new(self, text)
    }
}
