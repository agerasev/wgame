use wgpu::util::DeviceExt;

use crate::object::{Object, Vertices};

pub struct Polygon<'a> {
    pub(crate) vertex_count: u32,
    pub(crate) device: &'a wgpu::Device,
    pub(crate) vertices: &'a wgpu::Buffer,
    pub(crate) indices: &'a wgpu::Buffer,
    pub(crate) pipeline: &'a wgpu::RenderPipeline,
}

impl<'a> Object for Polygon<'a> {
    fn device(&self) -> &wgpu::Device {
        self.device
    }

    fn vertices(&self) -> Vertices<'_> {
        Vertices {
            count: 3 * (self.vertex_count - 2),
            vertex_buffer: self.vertices,
            index_buffer: self.indices,
        }
    }

    fn create_uniforms(&self, transformation: glam::Mat4) -> wgpu::BindGroup {
        let device = self.device();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(transformation.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM,
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
