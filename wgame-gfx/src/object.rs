use crate::{Instance, instance::Context};

pub trait InstanceVisitor<C: Context> {
    fn visit<T: Instance<Context = C>>(&mut self, instance: T);
}

pub trait Object {
    type Context: Context;
    fn visit_instances<V: InstanceVisitor<Self::Context>>(&self, visitor: &mut V);
}

impl<T: Object> Object for &'_ T {
    type Context = T::Context;
    fn visit_instances<V: InstanceVisitor<Self::Context>>(&self, visitor: &mut V) {
        (*self).visit_instances(visitor);
    }
}

impl<T: Object> Object for Option<T> {
    type Context = T::Context;
    fn visit_instances<V: InstanceVisitor<Self::Context>>(&self, visitor: &mut V) {
        if let Some(object) = self {
            object.visit_instances(visitor);
        }
    }
}
