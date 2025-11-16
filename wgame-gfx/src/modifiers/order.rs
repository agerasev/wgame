use std::hash::Hash;

use crate::{Camera, Instance, Resource};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Ordered<T> {
    pub inner: T,
    pub order: i64,
}

impl<T> Ordered<T> {
    pub fn new(inner: T, order: i64) -> Self {
        Ordered { inner, order }
    }
}

impl<T: Instance> Instance for Ordered<T> {
    type Resource = Ordered<T::Resource>;

    fn resource(&self) -> Self::Resource {
        Ordered::new(self.inner.resource(), self.order)
    }
    fn store(&self, camera: &Camera, storage: &mut <Self::Resource as Resource>::Storage) {
        self.inner.store(camera, storage);
    }
}

impl<T: Resource> Resource for Ordered<T> {
    type Storage = T::Storage;

    fn new_storage(&self) -> Self::Storage {
        self.inner.new_storage()
    }
    fn render(&self, storage: &Self::Storage, pass: &mut wgpu::RenderPass<'_>) {
        self.inner.render(storage, pass);
    }

    fn order(&self) -> i64 {
        self.order
    }
}
