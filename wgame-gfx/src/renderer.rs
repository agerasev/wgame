use crate::{
    Camera, Context, Instance,
    modifiers::{Colored, Transformed},
    types::{Color, Transform},
};

pub trait Renderer<C: Context = Camera> {
    fn insert<T: Instance<Context = C>>(&mut self, instance: T);
}

pub trait RendererExt: Renderer<Camera> {
    fn transform<X: Transform>(&mut self, xform: X) -> Transformed<&mut Self> {
        Transformed::new(self, xform)
    }
    fn color<C: Color>(&mut self, color: C) -> Colored<&mut Self> {
        Colored::new(self, color)
    }
}

impl<R: Renderer<Camera>> RendererExt for R {}
