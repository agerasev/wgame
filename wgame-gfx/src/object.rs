use crate::{
    Camera, Instance,
    modifiers::{Colored, Transformed},
    types::{Color, Transform},
};

pub trait InstanceVisitor {
    fn visit<T: Instance>(&mut self, camera: &Camera, instance: T);
}

pub trait Object {
    fn visit_instances<V: InstanceVisitor>(&self, camera: &Camera, visitor: &mut V);
}

impl<T: Object> Object for &'_ T {
    fn visit_instances<V: InstanceVisitor>(&self, camera: &Camera, visitor: &mut V) {
        (*self).visit_instances(camera, visitor);
    }
}

impl<T: Object> Object for Option<T> {
    fn visit_instances<V: InstanceVisitor>(&self, camera: &Camera, visitor: &mut V) {
        if let Some(object) = self {
            object.visit_instances(camera, visitor);
        }
    }
}

impl Object for () {
    fn visit_instances<V: InstanceVisitor>(&self, _: &Camera, _: &mut V) {}
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
