use glam::Mat2;
use wgpu::util::DeviceExt;

use crate::object::{Object, Uniforms, Vertices};

pub struct Triangle<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) vertex_buffer: &'a wgpu::Buffer,
    pub(crate) bind_group_layout: &'a wgpu::BindGroupLayout,
    pub(crate) pipeline: &'a wgpu::RenderPipeline,
}

impl<'a> Object for Triangle<'a> {
    fn vertices(&self) -> Vertices {
        Vertices {
            count: 3,
            buffer: self.vertex_buffer.clone(),
        }
    }

    fn uniforms(&self) -> Uniforms {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(Mat2::IDENTITY.as_ref()),
                usage: wgpu::BufferUsages::UNIFORM, // | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });

        Uniforms { buffer, bind_group }
    }

    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.pipeline.clone()
    }
}
