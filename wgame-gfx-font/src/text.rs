use std::cell::RefCell;

use glam::{Mat4, Quat, Vec3, Vec4};
use wgame_font::swash::{GlyphId, shape::ShapeContext};
use wgame_gfx::{Camera, Instance, Object, Resource};

use crate::{
    FontTexture,
    render::{GlyphInstance, TextInstance, TextResource},
};

thread_local! {
    static CONTEXT: RefCell<ShapeContext> = Default::default();
}

pub struct GlyphInfo {
    id: GlyphId,
    offset: f32,
}

pub struct Text {
    font: FontTexture,
    glyphs: Vec<GlyphInfo>,
}

impl Text {
    pub fn new(font: &FontTexture, text: &str) -> Option<Self> {
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
        if !glyphs.is_empty() {
            Some(Self {
                font: font.clone(),
                glyphs,
            })
        } else {
            None
        }
    }
}

impl Instance for Text {
    type Resource = TextResource;

    fn resource(&self) -> Self::Resource {
        TextResource::new(&self.font)
    }
    fn store(&self, camera: &Camera, storage: &mut <Self::Resource as Resource>::Storage) {
        let mut glyphs = Vec::with_capacity(self.glyphs.len());
        for glyph in &self.glyphs {
            let glyph_image = match self.font.glyph_info(glyph.id) {
                Some(x) => x,
                None => continue,
            };
            glyphs.push(GlyphInstance {
                xform: camera.view
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
                id: glyph.id,
            });
        }
        storage.instances.push(TextInstance {
            texture: self.font.clone(),
            glyphs,
            color: Vec4::ONE,
        });
    }
}

impl Object for Text {
    fn collect_into(&self, camera: &Camera, collector: &mut wgame_gfx::Collector) {
        collector.push(camera, self);
    }
}
