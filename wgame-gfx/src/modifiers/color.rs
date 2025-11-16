use half::f16;
use rgb::Rgba;

use crate::{Camera, Object, object::InstanceVisitor, types::Color};

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
    fn visit_instances<V: InstanceVisitor>(&self, camera: &Camera, collector: &mut V) {
        self.inner
            .visit_instances(&camera.color(self.color), collector);
    }
}
