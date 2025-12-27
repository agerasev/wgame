use std::rc::Rc;

use glam::{Mat4, Quat, Vec3};
use half::f16;
use rgb::Rgba;
use wgame_gfx::{
    Camera, Instance, InstanceVisitor, Object,
    modifiers::{Colorable, Transformable},
    types::{Color, Transform, color},
};
use wgame_typography::{TextMetrics, swash::GlyphId};

use crate::{
    FontTexture,
    render::{TextResource, TextStorage},
};

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone)]
pub struct Text {
    font: FontTexture,
    metrics: Rc<TextMetrics>,
    matrix: Mat4,
    color: Rgba<f16>,
    align: TextAlign,
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
            matrix: Mat4::from_scale(Vec3::splat(metrics.size().recip())),
            metrics: Rc::new(metrics),
            color: color::WHITE.to_rgba(),
            align: TextAlign::default(),
        }
    }

    pub fn text(&self) -> &str {
        self.metrics.text()
    }
    pub fn metrics(&self) -> &TextMetrics {
        &self.metrics
    }

    pub fn align(&self, align: TextAlign) -> Self {
        Self {
            align,
            ..self.clone()
        }
    }

    pub fn instance(&self) -> Option<TextInstance> {
        let width = self.metrics.width();
        let mut offset = match self.align {
            TextAlign::Left => 0.0,
            TextAlign::Center => -width / 2.0,
            TextAlign::Right => -width,
        };
        let mut glyphs = Vec::with_capacity(self.metrics.glyphs().len());
        for glyph in self.metrics.glyphs() {
            if let Some(glyph_image) = self.font.glyph_info(glyph.id) {
                glyphs.push(GlyphInstance {
                    xform: self.matrix
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
        if glyphs.is_empty() {
            None
        } else {
            Some(TextInstance {
                texture: self.font.clone(),
                glyphs: glyphs.into(),
                color: self.color,
            })
        }
    }
}

impl Transformable for Text {
    fn transform<X: Transform>(&self, xform: X) -> Self {
        Self {
            matrix: xform.to_mat4() * self.matrix,
            ..self.clone()
        }
    }
}

impl Colorable for Text {
    fn multiply_color<C: Color>(&self, color: C) -> Self {
        Self {
            color: self.color.multiply(color),
            ..self.clone()
        }
    }
}

#[derive(Clone)]
pub struct TextInstance {
    pub(crate) texture: FontTexture,
    pub(crate) glyphs: Rc<[GlyphInstance]>,
    pub(crate) color: Rgba<f16>,
}

pub(crate) struct GlyphInstance {
    pub(crate) xform: Mat4,
    pub(crate) id: GlyphId,
}

impl Instance for TextInstance {
    type Context = Camera;
    type Resource = TextResource;
    type Storage = TextStorage;

    fn resource(&self) -> Self::Resource {
        TextResource::new(&self.texture)
    }
    fn new_storage(&self) -> Self::Storage {
        TextStorage::new(self.resource())
    }
    fn store(&self, storage: &mut Self::Storage) {
        storage.instances.push(self.clone());
    }
}

impl Object for TextInstance {
    type Context = Camera;
    fn for_each_instance<V: InstanceVisitor<Self::Context>>(&self, visitor: &mut V) {
        visitor.visit(self);
    }
}

impl Object for Text {
    type Context = Camera;
    fn for_each_instance<V: InstanceVisitor<Self::Context>>(&self, visitor: &mut V) {
        if let Some(instance) = self.instance() {
            visitor.visit(&instance);
        }
    }
}
