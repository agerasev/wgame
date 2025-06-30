use glam::Mat4;

use crate::object::{Object, Vertices};

pub struct Transformed<T> {
    pub inner: T,
    pub transformation: Mat4,
}

impl<T: Object> Object for Transformed<T> {
    fn device(&self) -> &wgpu::Device {
        self.inner.device()
    }

    fn vertices(&self) -> Vertices<'_> {
        self.inner.vertices()
    }

    fn create_uniforms(&self, transformation: Mat4) -> wgpu::BindGroup {
        self.inner
            .create_uniforms(transformation * self.transformation)
    }

    fn pipeline(&self) -> &wgpu::RenderPipeline {
        self.inner.pipeline()
    }
}
