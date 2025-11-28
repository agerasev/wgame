use crate::{Instance, instance::Context};

pub trait Renderer<C: Context> {
    fn insert<T: Instance<Context = C>>(&mut self, instance: T);
}

pub trait Object {
    type Context: Context;
    fn draw<R: Renderer<Self::Context>>(&self, renderer: &mut R);
}

impl<T: Object> Object for &'_ T {
    type Context = T::Context;
    fn draw<R: Renderer<Self::Context>>(&self, renderer: &mut R) {
        (*self).draw(renderer);
    }
}

impl<T: Object> Object for Option<T> {
    type Context = T::Context;
    fn draw<R: Renderer<Self::Context>>(&self, renderer: &mut R) {
        if let Some(object) = self {
            object.draw(renderer);
        }
    }
}
