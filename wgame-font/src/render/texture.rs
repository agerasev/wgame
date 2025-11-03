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

impl PartialOrd for FontTexture {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for FontTexture {
    fn cmp(&self, other: &Self) -> Ordering {
        self.texture.as_ptr().cmp(&other.texture.as_ptr())
    }
}

impl PartialEq for FontTexture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.texture, &other.texture)
    }
}
impl Eq for FontTexture {}

impl Hash for FontTexture {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.texture.as_ptr().hash(state);
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
