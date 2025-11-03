mod atlas;

use core::cell::RefCell;
use std::rc::Rc;

use euclid::default::Size2D;
use swash::{GlyphId, scale::ScaleContext};
use wgame_image::{Atlas, AtlasImage};

use crate::Font;

pub use self::atlas::StyleAtlas;

thread_local! {
    static CONTEXT: RefCell<ScaleContext> = Default::default();
}

#[derive(Clone)]
pub struct Style {
    font: Font,
    size: f32,
    pub(crate) atlas: Rc<RefCell<StyleAtlas>>,
}

impl Style {
    pub fn new(atlas: &Atlas<u8>, font: &Font, size: f32) -> Self {
        let init_dim = ((4.0 * size).ceil().clamp(u32::MIN as f32, i32::MAX as f32) as u32)
            .next_power_of_two();
        let image = atlas.allocate(Size2D::new(init_dim, init_dim));
        Self {
            font: font.clone(),
            size,
            atlas: Rc::new(RefCell::new(StyleAtlas::new(image))),
        }
    }

    pub fn add_chars(&self, codepoints: impl IntoIterator<Item = impl Into<u32>>) {
        let font_ref = self.font.as_ref();
        self.add_glyphs(codepoints.into_iter().map(|c| font_ref.charmap().map(c)));
    }
    pub(crate) fn add_glyphs(&self, glyphs: impl IntoIterator<Item = GlyphId>) {
        let mut atlas = self.atlas.borrow_mut();

        CONTEXT.with_borrow_mut(|context| {
            let mut scaler = context
                .builder(self.font.as_ref())
                .size(self.size)
                .hint(false)
                .build();

            for glyph_id in glyphs {
                atlas.add_glyph(&mut scaler, glyph_id);
            }
        });
    }

    pub fn font(&self) -> &Font {
        &self.font
    }
    pub fn size(&self) -> f32 {
        self.size
    }
    pub fn image(&self) -> AtlasImage<u8> {
        self.atlas.borrow().image().clone()
    }
    pub fn atlas_svg(&self) -> Vec<u8> {
        self.atlas.borrow().debug_svg()
    }
}
