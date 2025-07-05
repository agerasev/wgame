use glam::Mat4;

use crate::types::Transform;

pub struct Vertices {
    pub count: u32,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
}

pub struct Uniforms {
    pub vertex: wgpu::BindGroup,
    pub fragment: wgpu::BindGroup,
}

pub trait Object {
    fn vertices(&self) -> Vertices;
    fn create_uniforms(&self, xform: Mat4) -> Uniforms;
    fn pipeline(&self) -> wgpu::RenderPipeline;
}

pub trait ObjectExt: Object + Sized {
    fn transform<T: Transform>(self, xform: T) -> Transformed<Self> {
        Transformed {
            inner: self,
            xform: xform.to_mat4(),
        }
    }
}

impl<T: Object> ObjectExt for T {}

pub struct Transformed<T> {
    pub inner: T,
    pub xform: Mat4,
}

impl<T: Object> Object for Transformed<T> {
    fn vertices(&self) -> Vertices {
        self.inner.vertices()
    }

    fn create_uniforms(&self, xform: Mat4) -> Uniforms {
        self.inner.create_uniforms(xform * self.xform)
    }

    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.inner.pipeline()
    }
}
