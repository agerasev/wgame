use core::cell::RefCell;

use swash::{GlyphId, shape::ShapeContext};

use wgame_gfx::Instance;

use crate::FontAtlas;

thread_local! {
    static CONTEXT: RefCell<ShapeContext> = Default::default();
}

pub struct GlyphInfo {
    id: GlyphId,
    offset: f32,
}

pub struct Text {
    atlas: FontAtlas,
    glyphs: Vec<GlyphInfo>,
}

impl Text {
    pub fn new(atlas: &FontAtlas, text: &str) -> Self {
        let font = atlas.font().clone();
        let glyphs = CONTEXT.with_borrow_mut(|context| {
            let mut shaper = context.builder(font.as_ref()).size(atlas.size()).build();
            shaper.add_str(text);
            let mut glyphs = Vec::new();
            shaper.shape_with(|cluster| {
                let mut offset = 0.0;
                for glyph in cluster.glyphs {
                    glyphs.push(GlyphInfo {
                        id: glyph.id,
                        offset,
                    });
                    offset += glyph.advance;
                }
            });
            glyphs
        });
        Self {
            atlas: atlas.clone(),
            glyphs,
        }
    }
}
/*
impl Instance for Text {
    type Renderer = TextLibrary;

    fn get_renderer(&self) -> Self::Renderer {
        unimplemented!()
    }
    fn store(
        &self,
        ctx: impl wgame_gfx::Context,
        storage: &mut <Self::Renderer as wgame_gfx::Renderer>::Storage,
    ) {
        unimplemented!()
    }
}
*/
