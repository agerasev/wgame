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
    fn render(
        &self,
        attachments: &wgpu::RenderPassDescriptor<'_>,
        encoder: &mut wgpu::CommandEncoder,
        xform: Mat4,
    ) {
        self.inner.render(attachments, encoder, xform * self.xform);
    }
}
