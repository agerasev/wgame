use core::cell::RefCell;

use glam::{Mat4, Quat, Vec3, Vec4};
use swash::{GlyphId, shape::ShapeContext};

use wgame_gfx::Instance;

use crate::{GlyphInstance, TextRenderer, TexturedFont};

thread_local! {
    static CONTEXT: RefCell<ShapeContext> = Default::default();
}

pub struct GlyphInfo {
    id: GlyphId,
    offset: f32,
}

pub struct Text {
    font: TexturedFont,
    glyphs: Vec<GlyphInfo>,
}

impl Text {
    pub fn new(font: &TexturedFont, text: &str) -> Self {
        let glyphs = CONTEXT.with_borrow_mut(|context| {
            let mut shaper = context
                .builder(font.font().as_ref())
                .size(font.size())
                .build();
            shaper.add_str(text);
            let mut glyphs = Vec::new();
            let mut offset = 0.0;
            shaper.shape_with(|cluster| {
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
        font.add_glyphs(glyphs.iter().map(|g| g.id));
        Self {
            font: font.clone(),
            glyphs,
        }
    }
}

impl Instance for Text {
    type Renderer = TextRenderer;

    fn get_renderer(&self) -> Self::Renderer {
        TextRenderer::new(&self.font)
    }
    fn store(
        &self,
        ctx: impl wgame_gfx::Context,
        storage: &mut <Self::Renderer as wgame_gfx::Renderer>::Storage,
    ) {
        let atlas = self.font.atlas.borrow();
        for glyph in &self.glyphs {
            let glyph_image = match atlas.get_glyph(glyph.id) {
                Some(x) => x,
                None => continue,
            };
            storage.instances.push(GlyphInstance {
                xform: ctx.view_matrix()
                    * Mat4::from_scale_rotation_translation(
                        Vec3::new(
                            glyph_image.placement.width as f32,
                            glyph_image.placement.height as f32,
                            1.0,
                        ),
                        Quat::IDENTITY,
                        Vec3::new(
                            glyph_image.placement.left as f32 + glyph.offset,
                            glyph_image.placement.top as f32,
                            0.0,
                        ),
                    ),
                tex_coord: Vec4::new(
                    glyph_image.location.x as f32,
                    glyph_image.location.y as f32,
                    glyph_image.location.width as f32,
                    glyph_image.location.height as f32,
                ),
                color: Vec4::ONE,
            });
        }
    }
}
