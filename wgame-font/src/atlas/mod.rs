mod inner;

use core::cell::RefCell;
use std::rc::Rc;

use euclid::default::Size2D;
use swash::{GlyphId, scale::ScaleContext};
use wgame_image::{Atlas, AtlasImage};

use self::inner::InnerAtlas;
use crate::Font;

pub(crate) use self::inner::GlyphImageInfo;

thread_local! {
    static CONTEXT: RefCell<ScaleContext> = Default::default();
}

#[derive(Clone)]
pub struct FontAtlas {
    font: Font,
    size: f32,
    pub(crate) atlas: Rc<RefCell<InnerAtlas>>,
}

impl FontAtlas {
    pub fn new(atlas: &Atlas<u8>, font: &Font, size: f32) -> Self {
        let init_dim = ((4.0 * size).ceil().clamp(u32::MIN as f32, i32::MAX as f32) as u32)
            .next_power_of_two();
        let image = atlas.allocate(Size2D::new(init_dim, init_dim));
        Self {
            font: font.clone(),
            size,
            atlas: Rc::new(RefCell::new(InnerAtlas::new(image))),
        }
    }

    pub fn add_chars(&self, codepoints: impl IntoIterator<Item = impl Into<u32>>) {
        let font_ref = self.font.as_ref();
        self.add_glyphs(codepoints.into_iter().map(|c| font_ref.charmap().map(c)));
    }
    pub fn add_glyphs(&self, glyphs: impl IntoIterator<Item = GlyphId>) {
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
    pub fn get_glyph(&self, glyph_id: GlyphId) -> Option<GlyphImageInfo> {
        self.atlas.borrow().get_glyph(glyph_id)
    }
    pub fn get_glyph_rect(&self, glyph_id: GlyphId) -> Option<Rect<u32>> {
        self.atlas.borrow().get_glyph(glyph_id)
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
