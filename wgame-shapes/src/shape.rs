use glam::{Affine2, Mat4, Vec2};

use wgame_gfx::{
    State, Texture, Transformed, Vertices,
    types::{Color, Transform},
};

use crate::{Textured, attributes::Attributes};

pub trait Shape<'a> {
    type Attributes: Attributes;

    fn state(&self) -> &State<'a>;
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

    fn state(&self) -> &State<'a> {
        T::state(self)
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
    fn transform<T: Transform>(self, xform: T) -> Transformed<Self> {
        Transformed {
            inner: self,
            xform: xform.to_mat4(),
        }
    }

    fn color<T: Color>(self, color: T) -> Textured<'a, Self> {
        let pixel = Texture::with_data(self.state(), (1, 1), &[color.to_rgba()]);
        self.texture(pixel)
    }

    fn gradient<T: Color>(self, colors: [[T; 2]; 2]) -> Textured<'a, Self> {
        let colors = colors.map(|row| row.map(|color| color.to_rgba()));
        let pixels_2x2 = Texture::with_data(self.state(), (2, 2), colors.as_flattened())
            .transform_coord(Affine2::from_scale_angle_translation(
                Vec2::new(0.5, 0.5),
                0.0,
                Vec2::new(0.25, 0.25),
            ));
        self.texture(pixels_2x2)
    }

    fn texture(self, texture: Texture<'a>) -> Textured<'a, Self> {
        Textured::new(self, texture)
    }
}

impl<'a, T: Shape<'a>> ShapeExt<'a> for T {}

impl<'a, T: Shape<'a>> Shape<'a> for Transformed<T> {
    type Attributes = T::Attributes;

    fn state(&self) -> &State<'a> {
        self.inner.state()
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
