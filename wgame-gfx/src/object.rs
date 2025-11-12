use crate::{
    Camera, Collector,
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

impl<T: Object> Object for Option<T> {
    fn collect_into(&self, camera: &Camera, collector: &mut Collector) {
        if let Some(object) = self {
            collector.push(camera, object)
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
