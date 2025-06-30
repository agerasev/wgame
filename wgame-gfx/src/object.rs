use glam::Mat4;

use crate::transform::Transformed;

pub struct Vertices<'a> {
    pub count: u32,
    pub buffer: &'a wgpu::Buffer,
}

pub struct Uniforms {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub trait Object {
    fn device(&self) -> &wgpu::Device;

    fn vertices(&self) -> Vertices<'_>;

    fn create_uniforms(&self, transformation: Mat4) -> wgpu::BindGroup;

    fn pipeline(&self) -> &wgpu::RenderPipeline;
}

pub trait ObjectExt: Sized {
    fn transform(self, transformation: Mat4) -> Transformed<Self> {
        Transformed {
            inner: self,
            transformation,
        }
    }
}

impl<T: Object> ObjectExt for T {}
