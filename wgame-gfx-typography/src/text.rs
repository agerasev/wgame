use glam::{Affine2, Mat4, Quat, Vec2, Vec3};
use half::f16;
use rgb::Rgba;
use wgame_gfx::{
    Camera, Instance, Object, Renderer, Resource,
    modifiers::Transformed,
    types::{Color, color},
};
use wgame_typography::{TextMetrics, swash::GlyphId};

use crate::{FontTexture, render::TextResource};

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

    pub fn instance(&self) -> Option<TextInstance> {
        let mut offset = 0.0;
        let mut glyphs = Vec::with_capacity(self.metrics.glyphs().len());
        for glyph in self.metrics.glyphs() {
            if let Some(glyph_image) = self.font.glyph_info(glyph.id) {
                glyphs.push(GlyphInstance {
                    xform: Mat4::from_scale_rotation_translation(
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
        if glyphs.is_empty() {
            None
        } else {
            Some(TextInstance {
                texture: self.font.clone(),
                glyphs,
                color: color::WHITE.to_rgba(),
            })
        }
    }
}

pub struct TextInstance {
    pub(crate) texture: FontTexture,
    pub(crate) glyphs: Vec<GlyphInstance>,
    pub(crate) color: Rgba<f16>,
}

pub(crate) struct GlyphInstance {
    pub(crate) xform: Mat4,
    pub(crate) id: GlyphId,
}

impl Instance for TextInstance {
    type Resource = TextResource;
    type Context = Camera;

    fn resource(&self) -> Self::Resource {
        TextResource::new(&self.texture)
    }
    fn store(&self, camera: &Camera, storage: &mut <Self::Resource as Resource>::Storage) {
        storage.instances.push(TextInstance {
            texture: self.texture.clone(),
            glyphs: self
                .glyphs
                .iter()
                .map(|glyph| GlyphInstance {
                    xform: camera.view
                        * Mat4::from_scale(Vec3::new(
                            1.0,
                            if camera.y_flip { -1.0 } else { 1.0 },
                            1.0,
                        ))
                        * glyph.xform,
                    id: glyph.id,
                })
                .collect(),
            color: self.color.multiply(camera.color),
        });
    }
}

impl Object for TextInstance {
    type Context = Camera;

    fn draw<R: Renderer<Self::Context>>(&self, renderer: &mut R) {
        renderer.insert(self);
    }
}

impl Object for Text {
    type Context = Camera;

    fn draw<R: Renderer<Self::Context>>(&self, renderer: &mut R) {
        if let Some(instance) = self.instance() {
            renderer.insert(instance);
        }
    }
}
