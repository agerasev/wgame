use glam::Mat4;
use wgame_gfx::{
    modifiers::Transformed,
    types::{Color, Transform},
};
use wgame_shader::Attribute;

use crate::{ShapesLibrary, Texture, Textured, renderer::VertexBuffers};

pub trait Shape {
    type Attribute: Attribute;

    fn state(&self) -> &ShapesLibrary;

    fn vertices(&self) -> VertexBuffers;
    fn uniforms(&self) -> Option<wgpu::BindGroup> {
        None
    }
    fn xform(&self) -> Mat4 {
        Mat4::IDENTITY
    }
    fn attribute(&self) -> Self::Attribute;
    fn pipeline(&self) -> wgpu::RenderPipeline;
}

impl<T: Shape> Shape for &T {
    type Attribute = T::Attribute;

    fn state(&self) -> &ShapesLibrary {
        T::state(self)
    }
    fn vertices(&self) -> VertexBuffers {
        T::vertices(self)
    }
    fn uniforms(&self) -> Option<wgpu::BindGroup> {
        T::uniforms(self)
    }
    fn xform(&self) -> Mat4 {
        T::xform(self)
    }
    fn attribute(&self) -> Self::Attribute {
        T::attribute(self)
    }
    fn pipeline(&self) -> wgpu::RenderPipeline {
        T::pipeline(self)
    }
}

pub trait ShapeExt: Shape + Sized {
    fn transform<T: Transform>(self, xform: T) -> Transformed<Self> {
        Transformed {
            inner: self,
            matrix: xform.to_mat4(),
        }
    }

    fn texture(self, texture: Texture) -> Textured<Self> {
        Textured::new(self, texture)
    }
    fn color(self, color: impl Color) -> Textured<Self> {
        let texture = self.state().white_texture.clone();
        Textured::new(self, texture).color(color)
    }
}

impl<T: Shape> ShapeExt for T {}

impl<T: Shape> Shape for Transformed<T> {
    type Attribute = T::Attribute;

    fn state(&self) -> &ShapesLibrary {
        self.inner.state()
    }
    fn vertices(&self) -> VertexBuffers {
        self.inner.vertices()
    }
    fn uniforms(&self) -> Option<wgpu::BindGroup> {
        self.inner.uniforms()
    }
    fn xform(&self) -> Mat4 {
        self.matrix * self.inner.xform()
    }
    fn attribute(&self) -> Self::Attribute {
        self.inner.attribute()
    }
    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.inner.pipeline()
    }
}
