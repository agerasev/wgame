use std::rc::Rc;

use glam::{Affine2, Mat4, Vec2, Vec4};
use wgpu::util::DeviceExt;

use crate::{Object, State, Transformed, object::Vertices};

use super::Texture;

pub trait Geometry<'a> {
    fn state(&self) -> &Rc<State<'a>>;
    fn vertices(&self) -> Vertices<'_>;
    fn pipeline(&self) -> &'_ wgpu::RenderPipeline;
}

pub trait GeometryExt<'a>: Geometry<'a> + Sized {
    fn transform(self, xform: Mat4) -> Transformed<Self> {
        Transformed { inner: self, xform }
    }

    fn color(self, rgba: Vec4) -> Textured<'a, Self> {
        let pixel = Texture::with_data(
            self.state(),
            (1, 1),
            wgpu::TextureFormat::Rgba32Float,
            bytemuck::cast_slice(&[rgba]),
        );
        self.texture(pixel)
    }

    fn gradient(self, colors: [[Vec4; 2]; 2]) -> Textured<'a, Self> {
        let pixels_2x2 = Texture::with_data(
            self.state(),
            (2, 2),
            wgpu::TextureFormat::Rgba32Float,
            bytemuck::cast_slice(&colors),
        )
        .transform(Affine2::from_scale_angle_translation(
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
    fn state(&self) -> &Rc<State<'a>> {
        self.inner.state()
    }

    fn vertices(&self) -> Vertices<'_> {
        self.inner.vertices()
    }

    fn pipeline(&self) -> &'_ wgpu::RenderPipeline {
        self.inner.pipeline()
    }
}

pub struct Textured<'a, T: Geometry<'a>> {
    pub geometry: T,
    pub texture: Texture<'a>,
}

impl<'a, T: Geometry<'a>> Object for Textured<'a, T> {
    fn vertices(&self) -> Vertices<'_> {
        self.geometry.vertices()
    }

    fn create_uniforms(&self, xform: Mat4) -> wgpu::BindGroup {
        let device = &self.geometry.state().device;
        let xform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("transform"),
            contents: bytemuck::cast_slice(xform.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let tex_xform = self.texture.xform.to_cols_array_2d();
        let text_xform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tex_transform"),
            contents: bytemuck::cast_slice(&tex_xform),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.texture.sampler),
                },
            ],
            label: None,
        })
    }

    fn pipeline(&self) -> &wgpu::RenderPipeline {
        self.geometry.pipeline()
    }
}
