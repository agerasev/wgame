use wgpu::util::DeviceExt;

use crate::object::{Object, Vertices};

pub struct Triangle<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) vertex_buffer: &'a wgpu::Buffer,
    pub(crate) pipeline: &'a wgpu::RenderPipeline,
}

impl<'a> Object for Triangle<'a> {
    fn device(&self) -> &wgpu::Device {
        self.device
    }

    fn vertices(&self) -> Vertices<'_> {
        Vertices {
            count: 3,
            buffer: self.vertex_buffer,
        }
    }

    fn create_uniforms(&self, transformation: glam::Mat4) -> wgpu::BindGroup {
        let device = self.device();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(transformation.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM, // | wgpu::BufferUsages::COPY_DST,
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.pipeline().get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        })
    }

    fn pipeline(&self) -> &wgpu::RenderPipeline {
        self.pipeline
    }
}
