use glam::Affine2;
use wgame_gfx::{Object, modifiers::Transformable, prelude::Colorable, types::Color};

use crate::{ShapesLibrary, Texture};

pub trait Shape: Transformable {
    fn library(&self) -> &ShapesLibrary;
}

pub trait ShapeFill: Shape {
    type Fill: Object + Textured + Colorable + Transformable;

    fn fill_texture(&self, texture: &Texture) -> Self::Fill;
    fn fill_color(&self, color: impl Color) -> Self::Fill {
        self.fill_texture(&self.library().white_texture.multiply_color(color))
    }
}

pub trait ShapeStroke: Shape {
    type Stroke: Object + Textured + Colorable + Transformable;

    fn stroke_texture(&self, line_width: f32, texture: &Texture) -> Self::Stroke;
    fn stroke_color(&self, line_width: f32, color: impl Color) -> Self::Stroke {
        self.stroke_texture(
            line_width,
            &self.library().white_texture.multiply_color(color),
        )
    }
}

pub trait Textured: Colorable + Sized {
    fn tranform_texcoord(&self, tex_xform: Affine2) -> Self;
}

#[macro_export]
macro_rules! impl_textured {
    ($self:ty, $texture:ident) => {
        impl $crate::Textured for $self {
            fn tranform_texcoord(&self, tex_xform: glam::Affine2) -> Self {
                Self {
                    $texture: self.$texture.transform_coord(tex_xform),
                    ..self.clone()
                }
            }
        }

        impl wgame_gfx::modifiers::Colorable for $self {
            fn multiply_color<C: wgame_gfx::types::Color>(&self, color: C) -> Self {
                Self {
                    $texture: self.$texture.multiply_color(color),
                    ..self.clone()
                }
            }
        }
    };
}
