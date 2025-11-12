use half::f16;
use rgb::Rgba;

use crate::{Context, Object, types::Color};

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

impl<T: Object> Object for Colored<T> {
    fn collect_into(&self, ctx: &Context, collector: &mut crate::Collector) {
        self.inner.collect_into(&ctx.color(self.color), collector);
    }
}
