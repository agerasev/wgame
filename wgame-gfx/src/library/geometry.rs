use glam::{Affine2, Mat4, Vec2};
use wgpu::util::DeviceExt;

use crate::{
    Object, SharedState, Transformed,
    object::{Uniforms, Vertices},
    types::{Color, Transform},
};

use super::Texture;

pub trait Geometry<'a> {
    fn state(&self) -> &SharedState<'a>;
    fn vertices(&self) -> Vertices;
    fn transformation(&self) -> Mat4;
    fn pipeline(&self) -> wgpu::RenderPipeline;
}

pub trait GeometryExt<'a>: Geometry<'a> + Sized {
    fn transform<T: Transform>(self, xform: T) -> Transformed<Self> {
        Transformed {
            inner: self,
            xform: xform.to_mat4(),
        }
    }

    fn color<T: Color>(self, color: T) -> Textured<'a, Self> {
        let pixel = Texture::with_data(
            self.state(),
            (1, 1),
            wgpu::TextureFormat::Rgba32Float,
            bytemuck::cast_slice(&[color.to_rgba()]),
        );
        self.texture(pixel)
    }

    fn gradient<T: Color>(self, colors: [[T; 2]; 2]) -> Textured<'a, Self> {
        let colors = colors.map(|row| row.map(|color| color.to_rgba()));
        let pixels_2x2 = Texture::with_data(
            self.state(),
            (2, 2),
            wgpu::TextureFormat::Rgba32Float,
            bytemuck::cast_slice(&colors),
        )
        .transform_coords(Affine2::from_scale_angle_translation(
            Vec2::new(0.5, 0.5),
            0.0,
            Vec2::new(0.25, 0.25),
        ));
        self.texture(pixels_2x2)
    }

    fn texture(self, texture: Texture<'a>) -> Textured<'a, Self> {
        Textured {
            geometry: self,
            texture,
        }
    }
}

impl<'a, T: Geometry<'a>> GeometryExt<'a> for T {}

impl<'a, T: Geometry<'a>> Geometry<'a> for Transformed<T> {
    fn state(&self) -> &SharedState<'a> {
        self.inner.state()
    }

    fn vertices(&self) -> Vertices {
        self.inner.vertices()
    }

    fn transformation(&self) -> Mat4 {
        self.xform * self.inner.transformation()
    }

    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.inner.pipeline()
    }
}

pub struct Textured<'a, T: Geometry<'a>> {
    pub geometry: T,
    pub texture: Texture<'a>,
}

impl<'a, T: Geometry<'a>> Object for Textured<'a, T> {
    fn vertices(&self) -> Vertices {
        self.geometry.vertices()
    }

    fn create_uniforms(&self, xform: Mat4) -> Uniforms {
        let device = &self.geometry.state().device;
        let final_xform = xform * self.geometry.transformation();
        let xform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("transform"),
            contents: bytemuck::cast_slice(final_xform.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let tex_xform = self.texture.xform.to_cols_array_2d();
        let text_xform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tex_transform"),
            contents: bytemuck::cast_slice(&tex_xform),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        Uniforms {
            vertex: device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.pipeline().get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: xform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: text_xform_buffer.as_entire_binding(),
                    },
                ],
                label: None,
            }),
            fragment: device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.pipeline().get_bind_group_layout(1),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.texture.sampler),
                    },
                ],
                label: None,
            }),
        }
    }

    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.geometry.pipeline()
    }
}
