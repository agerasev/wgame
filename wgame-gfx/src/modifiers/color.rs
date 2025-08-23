use half::f16;
use rgb::Rgba;

use crate::{Context, Instance, Resources, types::Color};

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

impl<T: Instance> Instance for Colored<T> {
    type Resources = T::Resources;

    fn get_resources(&self) -> Self::Resources {
        self.inner.get_resources()
    }
    fn store(&self, ctx: &Context, storage: &mut <Self::Resources as Resources>::Storage) {
        self.inner.store(&ctx.color(self.color), storage);
    }
}
