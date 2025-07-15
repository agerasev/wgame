use alloc::vec::Vec;

use glam::Mat4;

use crate::types::Transform;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Vertices {
    pub count: u32,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Model {
    pub index: i64,
    pub vertices: Vertices,
    pub uniforms: Vec<wgpu::BindGroup>,
    pub pipeline: wgpu::RenderPipeline,
}

pub trait Object {
    fn model(&self) -> Model;
    fn store_instance<D: Extend<u8>>(&self, xform: Mat4, buffer: &mut D);
}

impl<T: Object> Object for &'_ T {
    fn model(&self) -> Model {
        T::model(self)
    }
    fn store_instance<D: Extend<u8>>(&self, xform: Mat4, buffer: &mut D) {
        T::store_instance(self, xform, buffer);
    }
}

pub trait ObjectExt: Object + Sized {
    fn transform<T: Transform>(&self, xform: T) -> Transformed<&Self> {
        Transformed::new(self, xform)
    }
}

impl<T: Object> ObjectExt for T {}

pub struct Transformed<T> {
    pub inner: T,
    pub xform: Mat4,
}

impl<T: Object> Transformed<T> {
    pub fn new<X: Transform>(inner: T, xform: X) -> Self {
        Transformed {
            inner,
            xform: xform.to_mat4(),
        }
    }
}

impl<T: Object> Object for Transformed<T> {
    fn model(&self) -> Model {
        self.inner.model()
    }
    fn store_instance<D: Extend<u8>>(&self, xform: Mat4, buffer: &mut D) {
        self.inner.store_instance(xform * self.xform, buffer);
    }
}
