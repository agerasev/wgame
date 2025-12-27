use crate::{Context, Instance, Ordered};

pub trait InstanceVisitor<C: Context> {
    fn visit<T: Instance<Context = C>>(&mut self, instance: &T);
}

impl<C: Context, V: InstanceVisitor<C>> InstanceVisitor<C> for &mut V {
    fn visit<T: Instance<Context = C>>(&mut self, instance: &T) {
        (**self).visit(instance)
    }
}

pub trait Object {
    type Context: Context;
    fn for_each_instance<V: InstanceVisitor<Self::Context>>(&self, visitor: &mut V);

    fn order(&self, order: i32) -> Ordered<Self>
    where
        Self: Clone,
    {
        Ordered::new(self.clone(), order)
    }
}

impl<T: Object> Object for &T {
    type Context = T::Context;
    fn for_each_instance<V: InstanceVisitor<Self::Context>>(&self, visitor: &mut V) {
        (**self).for_each_instance(visitor);
    }
}
