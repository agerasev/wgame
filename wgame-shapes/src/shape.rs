use glam::Mat4;

use wgame_gfx::{
    State, Texture, Transformed, Vertices,
    types::{Color, Transform},
};

use crate::{Library, Textured, attributes::Attributes};

pub trait Shape<'a> {
    type Attributes: Attributes;

    fn library(&self) -> &Library<'a>;

    fn vertices(&self) -> Vertices;
    fn uniforms(&self) -> Option<wgpu::BindGroup> {
        None
    }
    fn xform(&self) -> Mat4 {
        Mat4::IDENTITY
    }
    fn attributes(&self) -> Self::Attributes;
    fn pipeline(&self) -> wgpu::RenderPipeline;
}

impl<'a, T: Shape<'a>> Shape<'a> for &T {
    type Attributes = T::Attributes;

    fn library(&self) -> &Library<'a> {
        T::library(self)
    }
    fn vertices(&self) -> Vertices {
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

pub trait ShapeExt<'a>: Shape<'a> + Sized {
    fn state(&self) -> &State<'a> {
        &self.library().0.state
    }

    fn transform<T: Transform>(self, xform: T) -> Transformed<Self> {
        Transformed {
            inner: self,
            xform: xform.to_mat4(),
        }
    }

    fn texture(self, texture: Texture<'a>) -> Textured<'a, Self> {
        Textured::new(self, texture)
    }
    fn color(self, color: impl Color) -> Textured<'a, Self> {
        let texture = self.library().0.white_texture.clone();
        Textured::new(self, texture).color(color)
    }
}

impl<'a, T: Shape<'a>> ShapeExt<'a> for T {}

impl<'a, T: Shape<'a>> Shape<'a> for Transformed<T> {
    type Attributes = T::Attributes;

    fn library(&self) -> &Library<'a> {
        self.inner.library()
    }
    fn vertices(&self) -> Vertices {
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
