use glam::Mat4;

use crate::{Camera, Instance, Object, Renderer, types::Transform};

pub trait Transformable {
    type Transformed;
    fn transform<X: Transform>(self, xform: X) -> Self::Transformed;
}

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

impl<R: Renderer<Camera>> Renderer<Camera> for Transformed<&mut R> {
    fn insert<T: Instance<Context = Camera>>(&mut self, instance: T) {
        self.inner.insert(Transformed::new(instance, self.matrix));
    }
}

impl<T: Object<Context = Camera>> Object for Transformed<T> {
    type Context = T::Context;

    fn draw<R: Renderer<T::Context>>(&self, visitor: &mut R) {
        self.inner.draw(&mut Transformed::new(visitor, self.matrix));
    }
}

impl<T: Instance<Context = Camera>> Instance for Transformed<T> {
    type Resource = T::Resource;
    type Context = T::Context;

    fn resource(&self) -> Self::Resource {
        self.inner.resource()
    }
    fn store(
        &self,
        context: &Self::Context,
        storage: &mut <Self::Resource as crate::Resource>::Storage,
    ) {
        self.inner.store(&context.transform(self.matrix), storage);
    }
}

impl<'a, T: Object<Context = Camera>> Transformable for &'a T {
    type Transformed = Transformed<&'a T>;

    fn transform<X: Transform>(self, xform: X) -> Self::Transformed {
        Transformed::new(self, xform)
    }
}

impl<'a, R: Renderer<Camera>> Transformable for &'a mut R {
    type Transformed = Transformed<&'a mut R>;

    fn transform<X: Transform>(self, xform: X) -> Self::Transformed {
        Transformed::new(self, xform)
    }
}
