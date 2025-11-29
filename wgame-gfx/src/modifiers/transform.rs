use glam::Mat4;

use crate::{Camera, Instance, Object, Renderer, Resource, types::Transform};

#[derive(Clone, Debug)]
pub struct Transformed<T> {
    pub inner: T,
    pub matrix: Mat4,
}

impl<T> Transformed<T> {
    pub fn new<X: Transform>(inner: T, xform: X) -> Self {
        Transformed {
            inner,
            matrix: xform.to_mat4(),
        }
    }
}

impl<R: Renderer> Renderer for Transformed<&mut R> {
    fn insert<T: Instance<Context = Camera>>(&mut self, instance: T) {
        self.inner.insert(Transformed::new(instance, self.matrix));
    }
}

impl<T: Object<Context = Camera>> Object for Transformed<T> {
    type Context = Camera;
    fn draw<R: Renderer<Self::Context>>(&self, renderer: &mut R) {
        self.inner
            .draw(&mut Transformed::new(renderer, self.matrix));
    }
}

impl<T: Instance<Context = Camera>> Instance for Transformed<T> {
    type Resource = T::Resource;
    type Context = Camera;

    fn resource(&self) -> Self::Resource {
        self.inner.resource()
    }
    fn store(&self, context: &Camera, storage: &mut <Self::Resource as Resource>::Storage) {
        self.inner.store(&context.transform(self.matrix), storage);
    }
}
