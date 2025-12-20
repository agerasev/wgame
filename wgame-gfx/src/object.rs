use crate::InstanceVisitor;

pub trait Object {
    fn for_each_instance<V: InstanceVisitor>(&self, visitor: &mut V);
}

impl<T: Object> Object for &'_ T {
    fn for_each_instance<V: InstanceVisitor>(&self, visitor: &mut V) {
        (*self).for_each_instance(visitor);
    }
}

impl<T: Object> Object for Option<T> {
    fn for_each_instance<V: InstanceVisitor>(&self, visitor: &mut V) {
        if let Some(object) = self {
            object.for_each_instance(visitor);
        }
    }
}
