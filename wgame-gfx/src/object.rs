use glam::Mat4;

use crate::types::Transform;

pub trait Object {
    fn render(
        &self,
        attachments: &wgpu::RenderPassDescriptor<'_>,
        encoder: &mut wgpu::CommandEncoder,
        xform: Mat4,
    );
}

impl<T: Object> Object for &'_ T {
    fn render(
        &self,
        attachments: &wgpu::RenderPassDescriptor<'_>,
        encoder: &mut wgpu::CommandEncoder,
        xform: Mat4,
    ) {
        T::render(*self, attachments, encoder, xform);
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
    fn render(
        &self,
        attachments: &wgpu::RenderPassDescriptor<'_>,
        encoder: &mut wgpu::CommandEncoder,
        xform: Mat4,
    ) {
        self.inner.render(attachments, encoder, xform * self.xform);
    }
}
