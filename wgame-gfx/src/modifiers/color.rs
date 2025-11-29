use half::f16;
use rgb::Rgba;

use crate::{Camera, Instance, Object, Renderer, Resource, types::Color};

#[derive(Clone, Debug)]
pub struct Colored<T> {
    pub inner: T,
    pub color: Rgba<f16>,
}

impl<T> Colored<T> {
    pub fn new<C: Color>(inner: T, color: C) -> Self {
        Colored {
            inner,
            color: color.to_rgba(),
        }
    }
}

impl<R: Renderer> Renderer for Colored<&mut R> {
    fn insert<T: Instance<Context = Camera>>(&mut self, instance: T) {
        self.inner.insert(Colored::new(instance, self.color));
    }
}

impl<T: Object<Context = Camera>> Object for Colored<T> {
    type Context = Camera;
    fn draw<R: Renderer<Self::Context>>(&self, renderer: &mut R) {
        self.inner.draw(&mut Colored::new(renderer, self.color));
    }
}

impl<T: Instance<Context = Camera>> Instance for Colored<T> {
    type Resource = T::Resource;
    type Context = Camera;

    fn resource(&self) -> Self::Resource {
        self.inner.resource()
    }
    fn store(&self, context: &Camera, storage: &mut <Self::Resource as Resource>::Storage) {
        self.inner.store(&context.color(self.color), storage);
    }
}
