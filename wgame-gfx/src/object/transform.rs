use glam::Mat2;

use crate::object::Object;

pub struct Transformed<T: Object> {
    pub inner: T,
    pub transformation: Mat2,
}

impl<T: Object> Object for Transformed<T> {
    fn device(&self) -> &wgpu::Device {
        self.inner.device()
    }

    fn vertices(&self) -> super::Vertices<'_> {
        self.inner.vertices()
    }

    fn tranformation(&self) -> glam::Mat2 {
        self.inner.tranformation() * self.transformation
    }

    fn pipeline(&self) -> &wgpu::RenderPipeline {
        self.inner.pipeline()
    }
}
