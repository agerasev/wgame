use glam::{Affine2, Mat4};
use half::f16;
use rgb::Rgba;
use wgame_gfx::{
    modifiers::{Colored, Transformed},
    types::{Color, Transform, color},
};
use wgame_shader::Attribute;

use crate::{ShapesLibrary, ShapesState, Texture, Textured, resource::VertexBuffers};

#[derive(Clone, Copy, Debug)]
pub struct ShapeContext {
    pub xform: Mat4,
    pub tex_xform: Affine2,
    pub color: Rgba<f16>,
}

impl Default for ShapeContext {
    fn default() -> Self {
        Self {
            xform: Mat4::IDENTITY,
            tex_xform: Affine2::IDENTITY,
            color: color::WHITE.to_rgba(),
        }
    }
}

pub trait Visitor {
    fn apply<T: Element>(&mut self, ctx: ShapeContext, element: &T);
}

pub trait Element {
    type Attribute: Attribute;

    fn state(&self) -> &ShapesState;
    fn vertices(&self) -> VertexBuffers;
    fn uniforms(&self) -> Option<wgpu::BindGroup> {
        None
    }
    fn attribute(&self) -> Self::Attribute;
    fn pipeline(&self) -> wgpu::RenderPipeline;
}

pub trait Shape {
    fn library(&self) -> &ShapesLibrary;
    fn visit<V: Visitor>(&self, ctx: ShapeContext, visitor: &mut V);
}

impl<T: Shape> Shape for &T {
    fn library(&self) -> &ShapesLibrary {
        T::library(self)
    }
    fn visit<V: Visitor>(&self, ctx: ShapeContext, visitor: &mut V) {
        T::visit(*self, ctx, visitor);
    }
}

pub trait ShapeExt: Shape + Sized {
    fn texture(self, texture: impl AsRef<Texture>) -> Textured<Self> {
        Textured::new(self, texture.as_ref().clone())
    }
    fn transform<T: Transform>(self, xform: T) -> Transformed<Self> {
        Transformed {
            inner: self,
            matrix: xform.to_mat4(),
        }
    }
    fn color(self, color: impl Color) -> Colored<Textured<Self>> {
        let texture = self.library().white_texture.clone();
        Colored::new(Textured::new(self, texture), color)
    }
}

impl<T: Shape> ShapeExt for T {}

impl<T: Shape> Shape for Transformed<T> {
    fn library(&self) -> &ShapesLibrary {
        self.inner.library()
    }
    fn visit<V: Visitor>(&self, ctx: ShapeContext, visitor: &mut V) {
        self.inner.visit(
            ShapeContext {
                xform: ctx.xform * self.matrix,
                ..ctx
            },
            visitor,
        );
    }
}
