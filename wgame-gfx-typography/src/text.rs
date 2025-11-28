use glam::{Affine2, Mat4, Quat, Vec2, Vec3, Vec4};
use wgame_gfx::{Camera, Instance, Object, Renderer, Resource, modifiers::Transformed};
use wgame_typography::TextMetrics;

use crate::{
    FontTexture,
    render::{GlyphInstance, TextInstance, TextResource},
};

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

pub struct Text {
    font: FontTexture,
    metrics: TextMetrics,
}

impl Text {
    pub fn new(font: &FontTexture, text: &str) -> Self {
        Self::from_metrics(
            font,
            TextMetrics::new(font.font(), font.size(), text.to_string()),
        )
    }
    pub fn from_metrics(font: &FontTexture, metrics: TextMetrics) -> Self {
        font.add_glyphs(metrics.glyphs().iter().map(|g| g.id));
        Self {
            font: font.clone(),
            metrics,
        }
    }

    pub fn text(&self) -> &str {
        self.metrics.text()
    }
    pub fn metrics(&self) -> &TextMetrics {
        &self.metrics
    }

    pub fn align(&self, align: TextAlign) -> Transformed<&Self> {
        let width = self.metrics.width();
        Transformed::new(
            self,
            Affine2::from_translation(Vec2::new(
                match align {
                    TextAlign::Left => 0.0,
                    TextAlign::Center => -width / 2.0,
                    TextAlign::Right => -width,
                },
                0.0,
            )),
        )
    }
}

impl Instance for Text {
    type Resource = TextResource;
    type Context = Camera;

    fn resource(&self) -> Self::Resource {
        TextResource::new(&self.font)
    }
    fn store(&self, camera: &Camera, storage: &mut <Self::Resource as Resource>::Storage) {
        let mut offset = 0.0;
        let mut glyphs = Vec::with_capacity(self.metrics.glyphs().len());
        for glyph in self.metrics.glyphs() {
            if let Some(glyph_image) = self.font.glyph_info(glyph.id) {
                glyphs.push(GlyphInstance {
                    xform: camera.view
                        * Mat4::from_scale(Vec3::new(
                            1.0,
                            if camera.y_flip { -1.0 } else { 1.0 },
                            1.0,
                        ))
                        * Mat4::from_scale_rotation_translation(
                            Vec3::new(
                                glyph_image.placement.width as f32,
                                glyph_image.placement.height as f32,
                                1.0,
                            ),
                            Quat::IDENTITY,
                            Vec3::new(
                                glyph_image.placement.left as f32 + offset,
                                glyph_image.placement.top as f32,
                                0.0,
                            ),
                        ),
                    id: glyph.id,
                });
            }
            offset += glyph.advance;
        }
        storage.instances.push(TextInstance {
            texture: self.font.clone(),
            glyphs,
            color: Vec4::ONE,
        });
    }
}

impl Object for Text {
    type Context = Camera;

    fn draw<R: Renderer<Self::Context>>(&self, renderer: &mut R) {
        if !self.metrics.glyphs().is_empty() {
            renderer.insert(self);
        }
    }
}
