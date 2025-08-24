use glam::Mat4;

use crate::{Context, Instance, Resources, types::Transform};

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
    type Resources = T::Resources;

    fn get_resources(&self) -> Self::Resources {
        self.inner.get_resources()
    }
    fn store(&self, ctx: &Context, storage: &mut <Self::Resources as Resources>::Storage) {
        self.inner.store(&ctx.transform(self.xform), storage);
    }
}
