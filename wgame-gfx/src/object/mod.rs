use glam::Mat2;
use wgpu::util::DeviceExt;

use crate::object::transform::Transformed;

pub mod transform;

pub struct Vertices<'a> {
    pub count: u32,
    pub buffer: &'a wgpu::Buffer,
}

pub struct Uniforms {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub trait Object: Sized {
    fn device(&self) -> &wgpu::Device;

    fn vertices(&self) -> Vertices<'_>;

    fn tranformation(&self) -> Mat2;
    fn transform(self, transformation: Mat2) -> Transformed<Self> {
        Transformed {
            inner: self,
            transformation,
        }
    }

    fn uniforms(&self) -> Uniforms {
        let device = self.device();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(self.tranformation().as_ref()),
            usage: wgpu::BufferUsages::UNIFORM, // | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.pipeline().get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });

        Uniforms { buffer, bind_group }
    }

    fn pipeline(&self) -> &wgpu::RenderPipeline;
}
