use std::iter;

use crate::{Context, Instance, InstanceVisitor, Object, Resource, Storage};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Ordered<T: ?Sized> {
    pub order: i32,
    pub inner: T,
}

impl<T> Ordered<T> {
    pub fn new(inner: T, order: i32) -> Self {
        Self { inner, order }
    }
}

impl<R: Resource> Resource for Ordered<R> {
    fn order(&self) -> impl Iterator<Item = i32> {
        iter::once(self.order).chain(self.inner.order())
    }
}

impl<S: Storage> Storage for Ordered<S> {
    type Context = S::Context;
    type Resource = Ordered<S::Resource>;
    type Renderer = S::Renderer;

    fn resource(&self) -> Self::Resource {
        Ordered::new(self.inner.resource(), self.order)
    }
    fn bake(&self) -> Self::Renderer {
        self.inner.bake()
    }
}

impl<T: Instance + ?Sized> Instance for Ordered<T> {
    type Context = T::Context;
    type Resource = Ordered<T::Resource>;
    type Storage = Ordered<T::Storage>;

    fn resource(&self) -> Self::Resource {
        Ordered::new(self.inner.resource(), self.order)
    }
    fn new_storage(&self) -> Self::Storage {
        Ordered::new(self.inner.new_storage(), self.order)
    }
    fn store(&self, storage: &mut Self::Storage) {
        self.inner.store(&mut storage.inner);
    }
}

impl<T: Object + ?Sized> Object for Ordered<T> {
    type Context = T::Context;
    fn for_each_instance<V: InstanceVisitor<Self::Context>>(&self, visitor: &mut V) {
        self.inner
            .for_each_instance(&mut Ordered::new(visitor, self.order));
    }
}

impl<C: Context, V: InstanceVisitor<C>> InstanceVisitor<C> for Ordered<V> {
    fn visit<T: Instance<Context = C>>(&mut self, instance: &T) {
        self.inner.visit(&Ordered::new(instance, self.order));
    }
}
