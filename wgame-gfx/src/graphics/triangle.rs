use glam::Mat2;

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

    fn pipeline(&self) -> &wgpu::RenderPipeline {
        self.pipeline
    }

    fn tranformation(&self) -> Mat2 {
        Mat2::IDENTITY
    }
}
