mod mapping;

use core::cell::RefCell;
use std::rc::Rc;

use image::GrayImage;
use swash::{GlyphId, scale::ScaleContext};

use crate::Font;

pub use self::mapping::FontAtlas;

thread_local! {
    static CONTEXT: RefCell<ScaleContext> = Default::default();
}

#[derive(Clone)]
pub struct RasterizedFont {
    font: Font,
    size: f32,
    pub(crate) atlas: Rc<RefCell<FontAtlas>>,
}

impl RasterizedFont {
    pub fn new(font: &Font, size: f32) -> Self {
        let init_dim = ((4.0 * size).ceil().clamp(u32::MIN as f32, i32::MAX as f32) as u32)
            .next_power_of_two();
        Self {
            font: font.clone(),
            size,
            atlas: Rc::new(RefCell::new(FontAtlas::new(init_dim))),
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
    pub fn image(&self) -> GrayImage {
        self.atlas.borrow().image().clone()
    }
    pub fn atlas_svg(&self) -> Vec<u8> {
        self.atlas.borrow().debug_svg()
    }
}
