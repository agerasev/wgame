use glam::Mat4;

use crate::{Context, Instance, Resource, types::Transform};

#[derive(Clone, Debug)]
pub struct Transformed<T> {
    pub inner: T,
    pub xform: Mat4,
}

impl<T> Transformed<T> {
    pub fn new<X: Transform>(inner: T, xform: X) -> Self {
        Transformed {
            inner,
            xform: xform.to_mat4(),
        }
    }
}

impl<T: Instance> Instance for Transformed<T> {
    type Resource = T::Resource;

    fn resource(&self) -> Self::Resource {
        self.inner.resource()
    }
    fn store(&self, ctx: &Context, storage: &mut <Self::Resource as Resource>::Storage) {
        self.inner.store(&ctx.transform(self.xform), storage);
    }
}
