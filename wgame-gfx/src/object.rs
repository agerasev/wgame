use crate::{Instance, instance::Context};

pub trait Visitor<C: Context> {
    fn add<T: Instance<Context = C>>(&mut self, instance: T);
}

pub trait Object {
    type Context: Context;
    fn draw<V: Visitor<Self::Context>>(&self, renderer: &mut V);
}

impl<T: Object> Object for &'_ T {
    type Context = T::Context;
    fn draw<V: Visitor<Self::Context>>(&self, renderer: &mut V) {
        (*self).draw(renderer);
    }
}

impl<T: Object> Object for Option<T> {
    type Context = T::Context;
    fn draw<V: Visitor<Self::Context>>(&self, renderer: &mut V) {
        if let Some(object) = self {
            object.draw(renderer);
        }
    }
}
