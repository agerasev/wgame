use glam::Mat4;

use wgame_gfx::{
    modifiers::Transformed,
    types::{Color, Transform},
};

use crate::{Library, Texture, Textured, attributes::Attributes, renderer::VertexBuffers};

pub trait Shape {
    type Attributes: Attributes;

    fn state(&self) -> &Library;

    fn vertices(&self) -> VertexBuffers;
    fn uniforms(&self) -> Option<wgpu::BindGroup> {
        None
    }
    fn xform(&self) -> Mat4 {
        Mat4::IDENTITY
    }
    fn attributes(&self) -> Self::Attributes;
    fn pipeline(&self) -> wgpu::RenderPipeline;
}

impl<T: Shape> Shape for &T {
    type Attributes = T::Attributes;

    fn state(&self) -> &Library {
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
    fn attributes(&self) -> Self::Attributes {
        T::attributes(self)
    }
    fn pipeline(&self) -> wgpu::RenderPipeline {
        T::pipeline(self)
    }
}

pub trait ShapeExt: Shape + Sized {
    fn transform<T: Transform>(self, xform: T) -> Transformed<Self> {
        Transformed {
            inner: self,
            xform: xform.to_mat4(),
        }
    }

    fn texture(self, texture: Texture) -> Textured<Self> {
        Textured::new(self, texture)
    }
    fn gradient<T: Color, const N: usize>(self, colors: [T; N]) -> Textured<Self> {
        let tex = self.state().gradient(colors);
        self.texture(tex)
    }
    fn gradient2<T: Color, const M: usize, const N: usize>(
        self,
        colors: [[T; M]; N],
    ) -> Textured<Self> {
        let tex = self.state().gradient2(colors);
        self.texture(tex)
    }
    fn color(self, color: impl Color) -> Textured<Self> {
        let texture = self.state().white_texture.clone();
        Textured::new(self, texture).color(color)
    }
}

impl<T: Shape> ShapeExt for T {}

impl<T: Shape> Shape for Transformed<T> {
    type Attributes = T::Attributes;

    fn state(&self) -> &Library {
        self.inner.state()
    }
    fn vertices(&self) -> VertexBuffers {
        self.inner.vertices()
    }
    fn uniforms(&self) -> Option<wgpu::BindGroup> {
        self.inner.uniforms()
    }
    fn xform(&self) -> Mat4 {
        self.xform * self.inner.xform()
    }
    fn attributes(&self) -> Self::Attributes {
        self.inner.attributes()
    }
    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.inner.pipeline()
    }
}
