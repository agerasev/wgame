use crate::{
    Camera, Collector, Instance,
    modifiers::{Colored, Transformed},
    types::{Color, Transform},
};

pub trait Object {
    fn collect_into(&self, camera: &Camera, collector: &mut Collector);
}

impl<T: Object> Object for &'_ T {
    fn collect_into(&self, camera: &Camera, collector: &mut Collector) {
        (*self).collect_into(camera, collector);
    }
}

impl<T: Instance> Object for Option<T> {
    fn collect_into(&self, camera: &Camera, collector: &mut Collector) {
        if let Some(instance) = self {
            collector.push(camera, instance)
        }
    }
}

impl Object for () {
    fn collect_into(&self, _: &Camera, _: &mut Collector) {}
}

pub trait ObjectExt: Object + Sized {
    fn transform<T: Transform>(&self, xform: T) -> Transformed<&Self> {
        Transformed::new(self, xform)
    }
    fn color<C: Color>(&self, color: C) -> Colored<&Self> {
        Colored::new(self, color)
    }
}

impl<T: Object> ObjectExt for T {}
