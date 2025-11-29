use crate::{
    Camera, Renderer,
    instance::Context,
    modifiers::{Colored, Transformed},
    types::{Color, Transform},
};

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

pub trait ObjectExt: Object<Context = Camera> {
    fn transform<X: Transform>(&self, xform: X) -> Transformed<&Self> {
        Transformed::new(self, xform)
    }
    fn color<C: Color>(&self, color: C) -> Colored<&Self> {
        Colored::new(self, color)
    }
}

impl<T: Object<Context = Camera>> ObjectExt for T {}
